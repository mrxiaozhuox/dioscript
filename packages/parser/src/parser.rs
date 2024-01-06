use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{
        escaped, tag, tag_no_case, take_till1, take_until, take_while, take_while1, take_while_m_n,
    },
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, space0, space1},
    combinator::{map, opt, peek, value},
    error::context,
    multi::{fold_many0, many0, separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{
    ast::{
        ConditionalStatement, DioAstStatement, FunctionCall, FunctionDefine, LoopStatement,
        ParamsType, UseStatement, FunctionName, VariableDefine,
    },
    element::{AstElement, AstElementContentType},
    types::AstValue,
};

enum AttributeType {
    Attribute((String, AstValue)),
    Content(String),
    Element(AstElement),
    InlineExpr(CalcExpr),
    Condition(ConditionalStatement),
    Loop(LoopStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalcExpr {
    Value(AstValue),
    LinkExpr(LinkExpr),
    Add(Box<CalcExpr>, Box<CalcExpr>),
    Sub(Box<CalcExpr>, Box<CalcExpr>),
    Mul(Box<CalcExpr>, Box<CalcExpr>),
    Div(Box<CalcExpr>, Box<CalcExpr>),
    Mod(Box<CalcExpr>, Box<CalcExpr>),
    Eq(Box<CalcExpr>, Box<CalcExpr>),
    Ne(Box<CalcExpr>, Box<CalcExpr>),
    Gt(Box<CalcExpr>, Box<CalcExpr>),
    Lt(Box<CalcExpr>, Box<CalcExpr>),
    Ge(Box<CalcExpr>, Box<CalcExpr>),
    Le(Box<CalcExpr>, Box<CalcExpr>),
    And(Box<CalcExpr>, Box<CalcExpr>),
    Or(Box<CalcExpr>, Box<CalcExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkExpr {
    pub this: AstValue,
    pub list: Vec<LinkExprPart>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkExprPart {
    Field(String),
    FunctionCall(FunctionCall),
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

    pub fn list(message: &str) -> IResult<&str, Vec<AstValue>> {
        context(
            "list",
            delimited(
                tag("["),
                separated_list0(
                    tag(","),
                    delimited(multispace0, TypeParser::parse, multispace0),
                ),
                delimited(opt(tag(",")), multispace0, tag("]")),
            ),
        )(message)
    }

    fn variable(message: &str) -> IResult<&str, String> {
        context(
            "variable",
            map(VariableParser::parse_var_name, |v| v.to_string()),
        )(message)
    }

    fn variable_index(message: &str) -> IResult<&str, (String, Box<AstValue>)> {
        context(
            "variable index",
            map(
                tuple((
                    VariableParser::parse_var_name,
                    delimited(tag("["), TypeParser::parse_index_type, tag("]")),
                )),
                |v| (v.0.to_string(), Box::new(v.1)),
            ),
        )(message)
    }

    fn dict(message: &str) -> IResult<&str, HashMap<String, AstValue>> {
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
                    |tuple_vec: Vec<(&str, AstValue)>| {
                        tuple_vec
                            .into_iter()
                            .map(|(k, v)| (String::from(k), v))
                            .collect()
                    },
                ),
                delimited(opt(tag(",")), multispace0, tag("}")),
            ),
        )(message)
    }

    fn tuple(message: &str) -> IResult<&str, (Box<AstValue>, Box<AstValue>)> {
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
                    |pair: (AstValue, AstValue)| (Box::new(pair.0), Box::new(pair.1)),
                ),
                tag(")"),
            ),
        )(message)
    }

    fn parse_index_type(message: &str) -> IResult<&str, AstValue> {
        context(
            "value",
            alt((
                map(TypeParser::number, AstValue::Number),
                map(TypeParser::string, |s| AstValue::String(String::from(s))),
                map(TypeParser::variable, AstValue::Variable),
            )),
        )(message)
    }

    pub fn parse(message: &str) -> IResult<&str, AstValue> {
        context(
            "value",
            alt((
                map(TypeParser::number, AstValue::Number),
                map(TypeParser::boolean, AstValue::Boolean),
                map(TypeParser::string, |s| AstValue::String(String::from(s))),
                map(TypeParser::list, AstValue::List),
                map(TypeParser::dict, AstValue::Dict),
                map(TypeParser::tuple, AstValue::Tuple),
                map(ElementParser::parse, AstValue::Element),
                map(FunctionParser::call, AstValue::FunctionCaller),
                map(FunctionParser::define, AstValue::FunctionDefine),
                map(TypeParser::variable_index, AstValue::VariableIndex),
                map(TypeParser::variable, AstValue::Variable),
            )),
        )(message)
    }
}

struct VariableParser;
impl VariableParser {
    fn parse_var_name(message: &str) -> IResult<&str, String> {
        context(
            "var name",
            map(
                pair(
                    alpha1,
                    take_while(|c: char| c.is_alphanumeric() || c == '_'),
                ),
                |(first, rest): (&str, &str)| format!("{}{}", first, rest).trim().to_string(),
            ),
        )(message)
    }

    fn parse(message: &str) -> IResult<&str, VariableDefine> {
        context(
            "variable",
            map(
                tuple((
                    opt(terminated(tag("let"), space1)),
                    terminated(Self::parse_var_name, delimited(space0, tag("="), space0)),
                    CalculateParser::expr,
                    tag(";"),
                )),
                |v| VariableDefine {
                    new: v.0.is_some(),
                    name: v.1.to_string(),
                    expr: v.2,
                },
            ),
        )(message)
    }
}

struct CalculateParser;
impl CalculateParser {
    fn factor(input: &str) -> IResult<&str, CalcExpr> {
        delimited(
            space0,
            alt((
                map(Self::link, |v| CalcExpr::LinkExpr(v)),
                map(TypeParser::parse, |v| CalcExpr::Value(v)),
                delimited(char('('), Self::expr, char(')')),
            )),
            space0,
        )(input)
    }

    fn link(input: &str) -> IResult<&str, LinkExpr> {
        delimited(
            space0,
            map(
                tuple((
                    terminated(
                        TypeParser::parse,
                        delimited(multispace0, tag("."), multispace0),
                    ),
                    separated_list1(
                        delimited(multispace0, tag("."), multispace0),
                        alt((
                            map(FunctionParser::call_single_name, |v| LinkExprPart::FunctionCall(v)),
                            map(
                                alt((
                                    VariableParser::parse_var_name,
                                    map(digit1, |v: &str| v.to_string()),
                                )),
                                |v| LinkExprPart::Field(v),
                            ),
                        )),
                    ),
                )),
                |v| LinkExpr {
                    this: v.0,
                    list: v.1,
                },
            ),
            space0,
        )(input)
    }

    fn bool(input: &str) -> IResult<&str, CalcExpr> {
        let (input, init) = Self::factor(input)?;
        delimited(
            space0,
            fold_many0(
                pair(
                    alt((
                        tag("=="),
                        tag("!="),
                        tag(">="),
                        tag("<="),
                        tag(">"),
                        tag("<"),
                    )),
                    Self::factor,
                ),
                move || init.clone(),
                |acc, (op, val)| match op {
                    "==" => CalcExpr::Eq(Box::new(acc), Box::new(val)),
                    "!=" => CalcExpr::Ne(Box::new(acc), Box::new(val)),
                    ">=" => CalcExpr::Ge(Box::new(acc), Box::new(val)),
                    "<=" => CalcExpr::Le(Box::new(acc), Box::new(val)),
                    ">" => CalcExpr::Gt(Box::new(acc), Box::new(val)),
                    "<" => CalcExpr::Lt(Box::new(acc), Box::new(val)),
                    _ => unreachable!(),
                },
            ),
            space0,
        )(input)
    }

    fn term(input: &str) -> IResult<&str, CalcExpr> {
        let (input, init) = Self::bool(input)?;
        delimited(
            space0,
            fold_many0(
                pair(alt((char('*'), char('/'), char('%'))), Self::bool),
                move || init.clone(),
                |acc, (op, val)| match op {
                    '*' => CalcExpr::Mul(Box::new(acc), Box::new(val)),
                    '/' => CalcExpr::Div(Box::new(acc), Box::new(val)),
                    '%' => CalcExpr::Mod(Box::new(acc), Box::new(val)),
                    _ => unreachable!(),
                },
            ),
            space0,
        )(input)
    }

    fn expr(input: &str) -> IResult<&str, CalcExpr> {
        let (input, init) = Self::term(input)?;
        delimited(
            space0,
            fold_many0(
                pair(alt((char('+'), char('-'))), Self::term),
                move || init.clone(),
                |acc, (op, val)| match op {
                    '+' => CalcExpr::Add(Box::new(acc), Box::new(val)),
                    '-' => CalcExpr::Sub(Box::new(acc), Box::new(val)),
                    _ => unreachable!(),
                },
            ),
            space0,
        )(input)
    }
}

struct FunctionParser;
impl FunctionParser {
    fn call(message: &str) -> IResult<&str, FunctionCall> {
        context(
            "function call",
            map(
                tuple((
                    terminated(
                        map(
                            separated_list1(tag("::"), VariableParser::parse_var_name),
                            |v| {
                                if v.len() > 1 {
                                    FunctionName::Namespace(v)
                                } else {
                                    FunctionName::Single(v.get(0).unwrap().to_string())
                                }
                            }
                        ),
                        tag("(")
                    ),
                    delimited(
                        space0,
                        separated_list0(tag(","), delimited(space0, TypeParser::parse, space0)),
                        space0,
                    ),
                    tag(")"),
                )),
                |(name, arguments, _)| FunctionCall { name, arguments },
            ),
        )(message)
    }

    fn call_single_name(message: &str) -> IResult<&str, FunctionCall> {
        context(
            "function call single",
            map(
                tuple((
                    terminated(
                        map(
                            VariableParser::parse_var_name,
                            |v| {
                                FunctionName::Single(v)
                            }
                        ),
                        tag("(")
                    ),
                    delimited(
                        space0,
                        separated_list0(tag(","), delimited(space0, TypeParser::parse, space0)),
                        space0,
                    ),
                    tag(")"),
                )),
                |(name, arguments, _)| FunctionCall { name, arguments },
            ),
        )(message)
    }

    fn define(message: &str) -> IResult<&str, FunctionDefine> {
        context(
            "function define",
            map(
                tuple((
                    pair(tag("fn"), space1),
                    opt(terminated(VariableParser::parse_var_name, space0)),
                    delimited(
                        tag("("),
                        alt((
                            map(preceded(tag("@"), VariableParser::parse_var_name), |name| {
                                ParamsType::Variable(name)
                            }),
                            map(
                                separated_list0(
                                    tag(","),
                                    delimited(space0, VariableParser::parse_var_name, space0),
                                ),
                                |v| ParamsType::List(v),
                            ),
                        )),
                        delimited(tag(")"), space0, tag("{")),
                    ),
                    delimited(multispace0, parse_rsx, pair(multispace0, tag("}"))),
                )),
                |(_, name, params, inner)| FunctionDefine {
                    name,
                    params,
                    inner,
                },
            ),
        )(message)
    }
}

struct StatementParser;
impl StatementParser {
    fn parse_if(message: &str) -> IResult<&str, ConditionalStatement> {
        context(
            "if statment",
            map(
                tuple((
                    pair(tag("if"), space1),
                    terminated(CalculateParser::expr, pair(space0, tag("{"))),
                    delimited(multispace0, parse_rsx, pair(multispace0, tag("}"))),
                    opt(delimited(
                        delimited(
                            space0,
                            tag("else"),
                            delimited(space0, tag("{"), multispace0),
                        ),
                        parse_rsx,
                        pair(multispace0, tag("}")),
                    )),
                )),
                |(_, cond, inner, otherwise)| ConditionalStatement {
                    condition: cond,
                    inner,
                    otherwise,
                },
            ),
        )(message)
    }
    fn parse_for(message: &str) -> IResult<&str, LoopStatement> {
        context(
            "for statement",
            map(
                tuple((
                    pair(tag("for"), space1),
                    pair(TypeParser::variable, pair(space1, tag("in"))),
                    delimited(space1, TypeParser::parse, pair(space0, tag("{"))),
                    delimited(multispace0, parse_rsx, pair(multispace0, tag("}"))),
                )),
                |(_, (var_name, _), iter, inner)| LoopStatement {
                    execute_type: crate::ast::LoopExecuteType::Iter {
                        iter,
                        var: var_name,
                    },
                    inner,
                },
            ),
        )(message)
    }
    fn parse_while(message: &str) -> IResult<&str, LoopStatement> {
        context(
            "while statement",
            map(
                tuple((
                    pair(tag("while"), space1),
                    terminated(CalculateParser::expr, pair(space0, tag("{"))),
                    delimited(multispace0, parse_rsx, pair(multispace0, tag("}"))),
                )),
                |(_, expr, inner)| LoopStatement {
                    execute_type: crate::ast::LoopExecuteType::Conditional(expr),
                    inner,
                },
            ),
        )(message)
    }
}

struct ModuleParser;
impl ModuleParser {

    fn  module_name_style(c: char) -> bool {
        matches!(c, 'a'..='z' |  '_')
    }

    fn parse_module_name(message: &str) -> IResult<&str, &str> {
        context("module name", take_while1(Self::module_name_style))(message)
    }

    fn parse_use(message: &str) -> IResult<&str, UseStatement> {
        context(
            "use statement",
            map(
                delimited(
                    pair(tag("use"), space1),
                    separated_list1(tag("::"), Self::parse_module_name),
                    pair(space0, tag(";"))
                ),
                |list| UseStatement(list.iter().map(|v| v.to_string()).collect())
            )
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

    fn parse(message: &str) -> IResult<&str, AstElement> {
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
                                    delimited(multispace0, StatementParser::parse_if, multispace0),
                                    |v| AttributeType::Condition(v),
                                ),
                                map(
                                    delimited(multispace0, StatementParser::parse_for, multispace0),
                                    |v| AttributeType::Loop(v),
                                ),
                                map(
                                    delimited(
                                        multispace0,
                                        StatementParser::parse_while,
                                        multispace0,
                                    ),
                                    |v| AttributeType::Loop(v),
                                ),
                                map(
                                    delimited(multispace0, CalculateParser::expr, multispace0),
                                    |v| AttributeType::InlineExpr(v),
                                ),
                            )),
                        ),
                        delimited(opt(tag(",")), multispace0, tag("}")),
                    ),
                ),
                |(name, attrs)| {
                    let mut attr: HashMap<String, AstValue> = HashMap::new();
                    let mut content = vec![];
                    for a in attrs {
                        match a {
                            AttributeType::Attribute((key, value)) => {
                                attr.insert(key, value);
                            }
                            AttributeType::Content(c) => {
                                content.push(AstElementContentType::Content(c));
                            }
                            AttributeType::Element(e) => {
                                content.push(AstElementContentType::Children(e));
                            }
                            AttributeType::InlineExpr(s) => {
                                content.push(AstElementContentType::InlineExpr(s));
                            }
                            AttributeType::Condition(c) => {
                                content.push(AstElementContentType::Condition(c));
                            }
                            AttributeType::Loop(l) => {
                                content.push(AstElementContentType::Loop(l));
                            }
                        }
                    }
                    let el = AstElement {
                        name: name.to_string(),
                        attributes: attr,
                        content,
                    };
                    el
                },
            ),
        )(message)
    }
}

fn comment(message: &str) -> IResult<&str, String> {
    context(
        "Comment",
        map(preceded(tag("//"), take_until("\n")), |comment: &str| {
            comment.trim().to_string()
        }),
    )(message)
}

pub(crate) fn parse_rsx(message: &str) -> IResult<&str, Vec<DioAstStatement>> {
    context(
        "AST Full",
        many0(delimited(
            multispace0,
            alt((
                map(comment, |v| DioAstStatement::LineComment(v)),
                map(VariableParser::parse, |v| {
                    DioAstStatement::VariableAss(v)
                }),
                map(
                    delimited(tag("return "), CalculateParser::expr, tag(";")),
                    |v| DioAstStatement::ReturnValue(v),
                ),
                map(
                    terminated(FunctionParser::call, pair(space0, tag(";"))),
                    |v| DioAstStatement::FunctionCall(v),
                ),
                map(StatementParser::parse_if, |v| {
                    DioAstStatement::IfStatement(v)
                }),
                map(StatementParser::parse_for, |v| {
                    DioAstStatement::LoopStatement(v)
                }),
                map(StatementParser::parse_while, |v| {
                    DioAstStatement::LoopStatement(v)
                }),
                map(FunctionParser::define, |v| {
                    DioAstStatement::FunctionDefine(v)
                }),
                map(ModuleParser::parse_use, |v| {
                    DioAstStatement::ModuleUse(v)
                }),
            )),
            multispace0,
        )),
    )(message)
}
