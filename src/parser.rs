use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, tag_no_case, take_till1, take_while1, take_while_m_n},
    character::complete::{alphanumeric1, multispace0, space0, space1},
    combinator::{map, opt, peek, value},
    error::context,
    multi::{fold_many1, many0, separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{
    ast::{
        ConditionalExpr, ConditionalSignal, ConditionalStatement, DioAstStatement, DioscriptAst,
    },
    types::Value,
};

enum AttributeType {
    Attribute((String, Value)),
    Content(String),
    Element(crate::element::Element),
    Reference(String),
}

struct TypeParser;
impl TypeParser {
    fn normal(message: &str) -> IResult<&str, &str> {
        take_till1(|c: char| c == '\\' || c == '"' || c.is_ascii_control())(message)
    }

    fn escapable(i: &str) -> IResult<&str, &str> {
        context(
            "escaped",
            alt((
                tag("\""),
                tag("\\"),
                tag("/"),
                tag("b"),
                tag("f"),
                tag("n"),
                tag("r"),
                tag("t"),
                TypeParser::parse_hex,
            )),
        )(i)
    }

    fn string_format(message: &str) -> IResult<&str, &str> {
        escaped(TypeParser::normal, '\\', TypeParser::escapable)(message)
    }

    fn parse_hex(message: &str) -> IResult<&str, &str> {
        context(
            "hex string",
            preceded(
                peek(tag("u")),
                take_while_m_n(5, 5, |c: char| c.is_ascii_hexdigit() || c == 'u'),
            ),
        )(message)
    }

    pub fn string(message: &str) -> IResult<&str, &str> {
        context(
            "string",
            alt((
                tag("\"\""),
                delimited(tag("\""), TypeParser::string_format, tag("\"")),
            )),
        )(message)
    }

    pub fn boolean(message: &str) -> IResult<&str, bool> {
        let parse_true = value(true, tag_no_case("true"));
        let parse_false = value(false, tag_no_case("false"));
        alt((parse_true, parse_false))(message)
    }

    pub fn number(message: &str) -> IResult<&str, f64> {
        double(message)
    }

    pub fn list(message: &str) -> IResult<&str, Vec<Value>> {
        context(
            "list",
            delimited(
                tag("["),
                separated_list0(
                    tag(","),
                    delimited(multispace0, TypeParser::parse, multispace0),
                ),
                tag("]"),
            ),
        )(message)
    }

    fn reference(message: &str) -> IResult<&str, String> {
        context(
            "reference",
            map(pair(tag("@"), VarParser::parse_var_name), |v| {
                v.1.to_string()
            }),
        )(message)
    }

    fn dict(message: &str) -> IResult<&str, HashMap<String, Value>> {
        context(
            "object",
            delimited(
                tag("{"),
                map(
                    separated_list0(
                        tag(","),
                        separated_pair(
                            delimited(multispace0, TypeParser::string, multispace0),
                            tag(":"),
                            delimited(multispace0, TypeParser::parse, multispace0),
                        ),
                    ),
                    |tuple_vec: Vec<(&str, Value)>| {
                        tuple_vec
                            .into_iter()
                            .map(|(k, v)| (String::from(k), v))
                            .collect()
                    },
                ),
                tag("}"),
            ),
        )(message)
    }

    fn tuple(message: &str) -> IResult<&str, (Box<Value>, Box<Value>)> {
        context(
            "tuple",
            delimited(
                tag("("),
                map(
                    separated_pair(
                        delimited(multispace0, TypeParser::parse, multispace0),
                        tag(","),
                        delimited(multispace0, TypeParser::parse, multispace0),
                    ),
                    |pair: (Value, Value)| (Box::new(pair.0), Box::new(pair.1)),
                ),
                tag(")"),
            ),
        )(message)
    }

    pub fn parse(message: &str) -> IResult<&str, Value> {
        context(
            "value",
            alt((
                map(TypeParser::number, Value::Number),
                map(TypeParser::boolean, Value::Boolean),
                map(TypeParser::string, |s| Value::String(String::from(s))),
                map(TypeParser::list, Value::List),
                map(TypeParser::dict, Value::Dict),
                map(TypeParser::tuple, Value::Tuple),
                map(TypeParser::reference, Value::Reference),
                map(ElementParser::parse, Value::Element),
            )),
        )(&message)
    }
}

struct VarParser;
impl VarParser {
    fn var_name_style(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
    }

    fn parse_var_name(message: &str) -> IResult<&str, &str> {
        context("var name", take_while1(Self::var_name_style))(message)
    }

    fn parse(message: &str) -> IResult<&str, (String, Value)> {
        context(
            "variable",
            map(
                tuple((
                    delimited(
                        tag("@"),
                        Self::parse_var_name,
                        delimited(space0, tag("="), space0),
                    ),
                    TypeParser::parse,
                    tag(";"),
                )),
                |v| (v.0.to_string(), v.1),
            ),
        )(message)
    }
}

struct StatementParser;
impl StatementParser {
    fn conditional(message: &str) -> IResult<&str, Vec<(ConditionalSignal, (bool, Value))>> {
        context(
            "conditional",
            alt((
                map(pair(opt(tag("!")), TypeParser::parse), |v| {
                    vec![(ConditionalSignal::None, (v.0.is_some(), v.1.clone()))]
                }),
                    fold_many1(
                        pair(
                            opt(alt((
                                delimited(multispace0, tag("=="), multispace0),
                                delimited(multispace0, tag("!="), multispace0),
                                delimited(multispace0, tag(">"), multispace0),
                            ))),
                            pair(opt(tag("!")), TypeParser::parse),
                        ),
                        Vec::new,
                        |mut arr: Vec<_>, (sign, (not, value)): (Option<&str>, (Option<&str>, Value))| {
                            println!("{:?}", value);
                            arr.push((
                                ConditionalSignal::from_string(sign.unwrap_or("").to_string()),
                                (not.is_some(), value.clone())
                            ));
                            arr
                        },
                    ),
            )),
        )(message)
    }
    fn parse_if(message: &str) -> IResult<&str, ConditionalStatement> {
        context(
            "if statment",
            map(
                tuple((
                    pair(tag("if"), space1),
                    terminated(Self::conditional, pair(space1, tag("{"))),
                    delimited(multispace0, parse_rsx, pair(multispace0, tag("}"))),
                )),
                |(_, cond, inner)| ConditionalStatement {
                    condition: ConditionalExpr(cond),
                    inner,
                },
            ),
        )(message)
    }
}

struct ElementParser;
impl ElementParser {
    fn parse_element_name(message: &str) -> IResult<&str, &str> {
        context("element name", alphanumeric1)(message)
    }

    fn attr_name_style(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-')
    }

    fn parse_attr_name(message: &str) -> IResult<&str, &str> {
        context("element name", take_while1(Self::attr_name_style))(message)
    }

    fn parse(message: &str) -> IResult<&str, crate::element::Element> {
        context(
            "element",
            map(
                pair(
                    terminated(ElementParser::parse_element_name, space0),
                    delimited(
                        tag("{"),
                        separated_list0(
                            tag(","),
                            alt((
                                map(
                                    separated_pair(
                                        delimited(
                                            multispace0,
                                            ElementParser::parse_attr_name,
                                            multispace0,
                                        ),
                                        tag(":"),
                                        delimited(multispace0, TypeParser::parse, multispace0),
                                    ),
                                    |v| AttributeType::Attribute((v.0.to_string(), v.1)),
                                ),
                                map(
                                    delimited(multispace0, ElementParser::parse, multispace0),
                                    |v| AttributeType::Element(v),
                                ),
                                map(
                                    delimited(multispace0, TypeParser::string, multispace0),
                                    |v| AttributeType::Content(v.to_string()),
                                ),
                                map(
                                    delimited(
                                        multispace0,
                                        pair(tag("@"), VarParser::parse_var_name),
                                        multispace0,
                                    ),
                                    |v| AttributeType::Reference(v.1.to_string()),
                                ),
                            )),
                        ),
                        delimited(opt(tag(",")), multispace0, tag("}")),
                    ),
                ),
                |(name, attrs)| {
                    let mut attr: HashMap<String, Value> = HashMap::new();
                    let mut content: String = String::new();
                    let mut children: Vec<crate::element::Element> = vec![];
                    for a in attrs {
                        match a {
                            AttributeType::Attribute((key, value)) => {
                                attr.insert(key, value);
                            }
                            AttributeType::Content(c) => {
                                content = c;
                            }
                            AttributeType::Element(e) => {
                                children.push(e);
                            }
                            AttributeType::Reference(_s) => {}
                        }
                    }
                    let el = crate::element::Element {
                        name: name.to_string(),
                        attributes: attr,
                        content,
                        children,
                    };
                    el
                },
            ),
        )(message)
    }
}

pub fn parse_rsx(message: &str) -> IResult<&str, Vec<DioAstStatement>> {
    context(
        "AST Full",
        many0(delimited(
            multispace0,
            alt((
                map(VarParser::parse, |v| {
                    DioAstStatement::ReferenceAss((v.0.to_string(), v.1.clone()))
                }),
                map(
                    delimited(tag("return "), TypeParser::parse, tag(";")),
                    |v| DioAstStatement::ReturnValue(v),
                ),
                map(StatementParser::parse_if, |v| {
                    DioAstStatement::IfStatement(v)
                }),
            )),
            multispace0,
        )),
    )(message)
}

#[test]
fn hello() {
    let v = parse_rsx(include_str!("../test.rsx"));
    println!("{:#?}", v.unwrap().1);
}
