use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, tag_no_case, take_till1, take_while1, take_while_m_n},
    character::complete::{alphanumeric1, multispace0, space0},
    combinator::{map, opt, peek, value},
    error::context,
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

use crate::types::Value;

#[allow(dead_code)]
enum AttributeType {
    Attribute((String, Value)),
    Content(String),
    Element(crate::element::Element),
    Comment,
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
                ElementParser::parse_hex,
            )),
        )(i)
    }

    fn string_format(message: &str) -> IResult<&str, &str> {
        escaped(ElementParser::normal, '\\', ElementParser::escapable)(message)
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
                delimited(tag("\""), ElementParser::string_format, tag("\"")),
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
            delimited(
                multispace0,
                alt((
                    map(TypeParser::number, Value::Number),
                    map(TypeParser::boolean, Value::Boolean),
                    map(TypeParser::string, |s| Value::String(String::from(s))),
                    map(TypeParser::list, Value::List),
                    map(TypeParser::dict, Value::Dict),
                    map(TypeParser::tuple, Value::Tuple),
                )),
                multispace0,
            ),
        )(&message)
    }
}

#[allow(dead_code)]
struct ElementParser;
impl ElementParser {
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
                ElementParser::parse_hex,
            )),
        )(i)
    }

    fn string_format(message: &str) -> IResult<&str, &str> {
        escaped(ElementParser::normal, '\\', ElementParser::escapable)(message)
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

    fn parse_string(message: &str) -> IResult<&str, &str> {
        context(
            "string",
            alt((
                tag("\"\""),
                delimited(tag("\""), ElementParser::string_format, tag("\"")),
            )),
        )(message)
    }

    fn parse_element_name(message: &str) -> IResult<&str, &str> {
        context("element name", alphanumeric1)(message)
    }

    fn attr_name_style(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-')
    }

    fn parse_attr_name(message: &str) -> IResult<&str, &str> {
        context("element name", take_while1(Self::attr_name_style))(message)
    }

    fn parse_element(message: &str) -> IResult<&str, crate::element::Element> {
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
                                    delimited(
                                        multispace0,
                                        ElementParser::parse_element,
                                        multispace0,
                                    ),
                                    |v| AttributeType::Element(v),
                                ),
                                map(
                                    delimited(
                                        multispace0,
                                        ElementParser::parse_string,
                                        multispace0,
                                    ),
                                    |v| AttributeType::Content(v.to_string()),
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
                            AttributeType::Comment => {}
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

#[test]
fn hello() {
    let v = ElementParser::parse_element(include_str!("../test.rsx"));
    println!("{:#?}", v.unwrap().1);
}
