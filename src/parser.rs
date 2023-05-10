use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_till1, take_while_m_n},
    character::complete::{alpha1, alphanumeric1, multispace0, space0},
    combinator::{map, peek},
    error::context,
    multi::separated_list0,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

enum AttributeType {
    Attribute((String, String)),
    Content(String),
    Element(crate::element::Element),
    Test(String),
}

struct ValueParser {}
impl ValueParser {
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
                ValueParser::parse_hex,
            )),
        )(i)
    }

    fn string_format(message: &str) -> IResult<&str, &str> {
        escaped(ValueParser::normal, '\\', ValueParser::escapable)(message)
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
                delimited(tag("\""), ValueParser::string_format, tag("\"")),
            )),
        )(message)
    }

    fn parse_element_name(message: &str) -> IResult<&str, &str> {
        context("element name", alphanumeric1)(message)
    }

    fn parse_element(message: &str) -> IResult<&str, crate::element::Element> {
        context(
            "element",
            map(
                pair(
                    terminated(ValueParser::parse_element_name, space0),
                    delimited(
                        tag("{"),
                        separated_list0(
                            tag(","),
                            alt((
                                map(
                                    separated_pair(
                                        delimited(
                                            multispace0,
                                            ValueParser::parse_string,
                                            multispace0,
                                        ),
                                        tag(":"),
                                        delimited(
                                            multispace0,
                                            ValueParser::parse_string,
                                            multispace0,
                                        ),
                                    ),
                                    |v| {
                                        AttributeType::Attribute((v.0.to_string(), v.1.to_string()))
                                    },
                                ),
                                map(
                                    delimited(multispace0, ValueParser::parse_element, multispace0),
                                    |v| AttributeType::Element(v),
                                ),
                                map(
                                    delimited(multispace0, ValueParser::parse_string, multispace0),
                                    |v| AttributeType::Content(v.to_string()),
                                ),
                            )),
                        ),
                        tag("}"),
                    ),
                ),
                |(name, attrs)| {
                    let mut attr: HashMap<String, String> = HashMap::new();
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
                            AttributeType::Test(e) => {
                                println!("test value: {e}");
                            }
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
    let v = ValueParser::parse_element(include_str!("../test.rsx"));
    println!("{:?}", v);
}
