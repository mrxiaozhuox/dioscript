use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{
        escaped_transform, tag, tag_no_case, take, take_till1, take_while, take_while1,
    },
    character::complete::{
        alpha1, alphanumeric1, char, digit1, multispace0, not_line_ending, space0, space1,
    },
    combinator::{cut, map, opt, peek, value},
    error::{context, VerboseError},
    multi::{fold_many0, many0, separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{
    ast::{
        ConditionalStatement, DioAstStatement, FunctionCall, FunctionDefine, FunctionName,
        LoopStatement, ParamsType, UseStatement, VariableDefine,
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

type ParserResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

struct TypeParser;
impl TypeParser {
    // string

    pub fn string(i: &str) -> ParserResult<String> {
        let string_content = escaped_transform(
            take_till1(|c: char| c == '\\' || c == '"' || c.is_ascii_control()),
            '\\',
            alt((
                value("\"", tag("\"")),
                value("\\", tag("\\")),
                value("/", tag("/")),
                value("\u{0008}", tag("b")),
                value("\u{000C}", tag("f")),
                value("\n", tag("n")),
                value("\r", tag("r")),
                value("\t", tag("t")),
                map(take(1u8), |c: &str| c),
            )),
        );

        context("string", delimited(char('"'), string_content, char('"')))(i)
    }

    // boolean
    pub fn boolean(message: &str) -> ParserResult<bool> {
        let parse_true = value(true, tag_no_case("true"));
        let parse_false = value(false, tag_no_case("false"));
        alt((parse_true, parse_false))(message)
    }

    pub fn number(message: &str) -> ParserResult<f64> {
        double(message)
    }

    pub fn list(message: &str) -> ParserResult<Vec<CalcExpr>> {
        context(
            "list",
            delimited(
                tag("["),
                separated_list0(
                    tag(","),
                    delimited(multispace0, CalculateParser::expr, multispace0),
                ),
                delimited(opt(tag(",")), multispace0, tag("]")),
            ),
        )(message)
    }

    fn variable(message: &str) -> ParserResult<String> {
        context(
            "variable",
            map(VariableParser::parse_var_name, |v| v.to_string()),
        )(message)
    }

    fn variable_index(message: &str) -> ParserResult<(String, Box<CalcExpr>)> {
        context(
            "variable index",
            map(
                tuple((
                    VariableParser::parse_var_name,
                    delimited(tag("["), CalculateParser::expr, tag("]")),
                )),
                |v| (v.0.to_string(), Box::new(v.1)),
            ),
        )(message)
    }

    fn dict(message: &str) -> ParserResult<HashMap<String, CalcExpr>> {
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
                            delimited(multispace0, CalculateParser::expr, multispace0),
                        ),
                    ),
                    |tuple_vec: Vec<(String, CalcExpr)>| tuple_vec.into_iter().collect(),
                ),
                delimited(opt(tag(",")), multispace0, tag("}")),
            ),
        )(message)
    }

    fn tuple(message: &str) -> ParserResult<(Box<CalcExpr>, Box<CalcExpr>)> {
        context(
            "tuple",
            delimited(
                tag("("),
                map(
                    separated_pair(
                        delimited(multispace0, CalculateParser::expr, multispace0),
                        tag(","),
                        delimited(multispace0, CalculateParser::expr, multispace0),
                    ),
                    |pair: (CalcExpr, CalcExpr)| (Box::new(pair.0), Box::new(pair.1)),
                ),
                tag(")"),
            ),
        )(message)
    }

    pub fn parse(message: &str) -> ParserResult<AstValue> {
        context(
            "value",
            alt((
                map(TypeParser::number, AstValue::Number),
                map(TypeParser::boolean, AstValue::Boolean),
                map(TypeParser::string, AstValue::String),
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
    fn parse_var_name(message: &str) -> ParserResult<String> {
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

    fn parse(message: &str) -> ParserResult<VariableDefine> {
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
    fn factor(input: &str) -> ParserResult<CalcExpr> {
        delimited(
            space0,
            alt((
                map(Self::link, CalcExpr::LinkExpr),
                map(TypeParser::parse, CalcExpr::Value),
                delimited(char('('), Self::expr, char(')')),
            )),
            space0,
        )(input)
    }

    fn link(input: &str) -> ParserResult<LinkExpr> {
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
                            map(FunctionParser::call_single_name, LinkExprPart::FunctionCall),
                            map(
                                alt((
                                    VariableParser::parse_var_name,
                                    map(digit1, |v: &str| v.to_string()),
                                )),
                                LinkExprPart::Field,
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

    fn term(input: &str) -> ParserResult<CalcExpr> {
        let (input, init) = Self::factor(input)?;
        fold_many0(
            pair(
                delimited(space0, alt((char('*'), char('/'), char('%'))), space0),
                Self::factor,
            ),
            move || init.clone(),
            |acc, (op, val)| match op {
                '*' => CalcExpr::Mul(Box::new(acc), Box::new(val)),
                '/' => CalcExpr::Div(Box::new(acc), Box::new(val)),
                '%' => CalcExpr::Mod(Box::new(acc), Box::new(val)),
                _ => unreachable!(),
            },
        )(input)
    }

    fn add_sub(input: &str) -> ParserResult<CalcExpr> {
        let (input, init) = Self::term(input)?;
        fold_many0(
            pair(
                delimited(space0, alt((char('+'), char('-'))), space0),
                Self::term,
            ),
            move || init.clone(),
            |acc, (op, val)| match op {
                '+' => CalcExpr::Add(Box::new(acc), Box::new(val)),
                '-' => CalcExpr::Sub(Box::new(acc), Box::new(val)),
                _ => unreachable!(),
            },
        )(input)
    }

    fn comparison(input: &str) -> ParserResult<CalcExpr> {
        let (input, init) = Self::add_sub(input)?;
        fold_many0(
            pair(
                delimited(
                    space0,
                    alt((
                        tag("=="),
                        tag("!="),
                        tag(">="),
                        tag("<="),
                        tag(">"),
                        tag("<"),
                    )),
                    space0,
                ),
                Self::add_sub,
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
        )(input)
    }

    fn logical_and(input: &str) -> ParserResult<CalcExpr> {
        let (input, init) = Self::comparison(input)?;
        fold_many0(
            pair(delimited(space0, tag("&&"), space0), Self::comparison),
            move || init.clone(),
            |acc, (_, val)| CalcExpr::And(Box::new(acc), Box::new(val)),
        )(input)
    }

    fn logical_or(input: &str) -> ParserResult<CalcExpr> {
        let (input, init) = Self::logical_and(input)?;
        fold_many0(
            pair(delimited(space0, tag("||"), space0), Self::logical_and),
            move || init.clone(),
            |acc, (_, val)| CalcExpr::Or(Box::new(acc), Box::new(val)),
        )(input)
    }

    fn expr(input: &str) -> ParserResult<CalcExpr> {
        context("calculate expr", Self::logical_or)(input)
    }
}

struct FunctionParser;
impl FunctionParser {
    fn call(message: &str) -> ParserResult<FunctionCall> {
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
                                    FunctionName::Single(v.first().unwrap().to_string())
                                }
                            },
                        ),
                        tag("("),
                    ),
                    delimited(
                        space0,
                        separated_list0(tag(","), delimited(space0, CalculateParser::expr, space0)),
                        space0,
                    ),
                    tag(")"),
                )),
                |(name, arguments, _)| FunctionCall { name, arguments },
            ),
        )(message)
    }

    fn call_single_name(message: &str) -> ParserResult<FunctionCall> {
        context(
            "function call single",
            map(
                tuple((
                    terminated(
                        map(VariableParser::parse_var_name, FunctionName::Single),
                        tag("("),
                    ),
                    delimited(
                        space0,
                        separated_list0(tag(","), delimited(space0, CalculateParser::expr, space0)),
                        space0,
                    ),
                    tag(")"),
                )),
                |(name, arguments, _)| FunctionCall { name, arguments },
            ),
        )(message)
    }

    fn define(message: &str) -> ParserResult<FunctionDefine> {
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
                                ParamsType::List,
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
    fn parse_if(message: &str) -> ParserResult<ConditionalStatement> {
        context(
            "if statment",
            map(
                tuple((
                    context("if keyword", pair(tag("if"), space1)),
                    context(
                        "condition expr",
                        terminated(CalculateParser::expr, pair(space0, tag("{"))),
                    ),
                    context(
                        "if body",
                        delimited(multispace0, parse_rsx, pair(multispace0, tag("}"))),
                    ),
                    opt(delimited(
                        context(
                            "else keyword",
                            delimited(
                                space0,
                                tag("else"),
                                delimited(space0, tag("{"), multispace0),
                            ),
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
    fn parse_for(message: &str) -> ParserResult<LoopStatement> {
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
    fn parse_while(message: &str) -> ParserResult<LoopStatement> {
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
    fn module_name_style(c: char) -> bool {
        matches!(c, 'a'..='z' | '_')
    }

    fn parse_module_name(message: &str) -> ParserResult<&str> {
        context("module name", take_while1(Self::module_name_style))(message)
    }

    fn parse_use(message: &str) -> ParserResult<UseStatement> {
        context(
            "use statement",
            map(
                delimited(
                    pair(tag("use"), space1),
                    separated_list1(tag("::"), Self::parse_module_name),
                    pair(space0, tag(";")),
                ),
                |list| UseStatement(list.iter().map(|v| v.to_string()).collect()),
            ),
        )(message)
    }
}

struct ElementParser;
impl ElementParser {
    fn parse_element_name(message: &str) -> ParserResult<&str> {
        context("element name", alphanumeric1)(message)
    }

    fn attr_name_style(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-')
    }

    fn parse_attr_name(message: &str) -> ParserResult<&str> {
        context("element name", take_while1(Self::attr_name_style))(message)
    }

    fn parse(message: &str) -> ParserResult<AstElement> {
        context(
            "element",
            map(
                pair(
                    context(
                        "element name",
                        terminated(ElementParser::parse_element_name, multispace0),
                    ),
                    context(
                        "attributes",
                        delimited(
                            tag("{"),
                            map(
                                pair(
                                    many0(alt((
                                        terminated(
                                            alt((
                                                map(
                                                    context(
                                                        "attribute pair",
                                                        separated_pair(
                                                            delimited(
                                                                multispace0,
                                                                ElementParser::parse_attr_name,
                                                                multispace0,
                                                            ),
                                                            tag(":"),
                                                            delimited(
                                                                multispace0,
                                                                TypeParser::parse,
                                                                multispace0,
                                                            ),
                                                        ),
                                                    ),
                                                    |v| {
                                                        AttributeType::Attribute((
                                                            v.0.to_string(),
                                                            v.1,
                                                        ))
                                                    },
                                                ),
                                                // include type parser and calculate
                                                map(
                                                    delimited(
                                                        multispace0,
                                                        CalculateParser::expr,
                                                        multispace0,
                                                    ),
                                                    AttributeType::InlineExpr,
                                                ),
                                            )),
                                            tag(","),
                                        ),
                                        alt((
                                            map(
                                                delimited(
                                                    multispace0,
                                                    ElementParser::parse,
                                                    multispace0,
                                                ),
                                                AttributeType::Element,
                                            ),
                                            map(
                                                delimited(
                                                    multispace0,
                                                    StatementParser::parse_if,
                                                    multispace0,
                                                ),
                                                AttributeType::Condition,
                                            ),
                                            map(
                                                delimited(
                                                    multispace0,
                                                    StatementParser::parse_for,
                                                    multispace0,
                                                ),
                                                AttributeType::Loop,
                                            ),
                                            map(
                                                delimited(
                                                    multispace0,
                                                    StatementParser::parse_while,
                                                    multispace0,
                                                ),
                                                AttributeType::Loop,
                                            ),
                                        )),
                                    ))),
                                    opt(alt((
                                        map(
                                            separated_pair(
                                                delimited(
                                                    multispace0,
                                                    ElementParser::parse_attr_name,
                                                    multispace0,
                                                ),
                                                tag(":"),
                                                delimited(
                                                    multispace0,
                                                    TypeParser::parse,
                                                    multispace0,
                                                ),
                                            ),
                                            |v| AttributeType::Attribute((v.0.to_string(), v.1)),
                                        ),
                                        map(
                                            delimited(
                                                multispace0,
                                                CalculateParser::expr,
                                                multispace0,
                                            ),
                                            AttributeType::InlineExpr,
                                        ),
                                        map(
                                            delimited(multispace0, TypeParser::string, multispace0),
                                            |v| AttributeType::Content(v.to_string()),
                                        ),
                                        map(
                                            delimited(
                                                multispace0,
                                                ElementParser::parse,
                                                multispace0,
                                            ),
                                            AttributeType::Element,
                                        ),
                                        map(
                                            delimited(
                                                multispace0,
                                                StatementParser::parse_if,
                                                multispace0,
                                            ),
                                            AttributeType::Condition,
                                        ),
                                        map(
                                            delimited(
                                                multispace0,
                                                StatementParser::parse_for,
                                                multispace0,
                                            ),
                                            AttributeType::Loop,
                                        ),
                                        map(
                                            delimited(
                                                multispace0,
                                                StatementParser::parse_while,
                                                multispace0,
                                            ),
                                            AttributeType::Loop,
                                        ),
                                    ))),
                                ),
                                |(mut attrs, last_attr)| {
                                    if let Some(attr) = last_attr {
                                        attrs.push(attr);
                                    }
                                    attrs
                                },
                            ),
                            tag("}"),
                        ),
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
                    AstElement {
                        name: name.to_string(),
                        attributes: attr,
                        content,
                    }
                },
            ),
        )(message)
    }
}

fn comment(message: &str) -> ParserResult<String> {
    context(
        "Comment",
        map(preceded(tag("//"), not_line_ending), |comment: &str| {
            comment.trim().to_string()
        }),
    )(message)
}

pub(crate) fn parse_rsx(message: &str) -> ParserResult<Vec<DioAstStatement>> {
    context(
        "AST Full",
        many0(delimited(
            multispace0,
            alt((
                map(comment, DioAstStatement::LineComment),
                map(VariableParser::parse, DioAstStatement::VariableAss),
                map(
                    delimited(cut(tag("return ")), CalculateParser::expr, tag(";")),
                    DioAstStatement::ReturnValue,
                ),
                map(
                    terminated(FunctionParser::call, pair(space0, tag(";"))),
                    DioAstStatement::FunctionCall,
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
                map(ModuleParser::parse_use, DioAstStatement::ModuleUse),
            )),
            multispace0,
        )),
    )(message)
}
