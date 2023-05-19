use std::str::FromStr;

use crate::{parser::parse_rsx, types::Value};

#[derive(Debug, Clone, PartialEq)]
pub struct DioscriptAst {
    pub stats: Vec<DioAstStatement>,
}

impl DioscriptAst {
    pub fn to_ast(message: &str) -> Self {
        let v = parse_rsx(message).ok().unwrap().1;
        Self { stats: v }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DioAstStatement {
    ReferenceAss((String, Value)),
    ReturnValue(Value),
    IfStatement(ConditionalStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalStatement {
    pub condition: ConditionalExpr,
    pub inner: Vec<DioAstStatement>,
    pub otherwise: Option<Vec<DioAstStatement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpr(pub Vec<(ConditionalMark, SubExpr)>);

#[derive(Debug, Clone, PartialEq)]
pub enum SubExpr {
    Single((bool, Value)),
    Pair(ConditionalExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalculateMark {
    None,
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl ToString for CalculateMark {
    fn to_string(&self) -> String {
        match self {
            CalculateMark::None => "none".to_string(),
            CalculateMark::Plus => "+".to_string(),
            CalculateMark::Minus => "-".to_string(),
            CalculateMark::Multiply => "*".to_string(),
            CalculateMark::Divide => "/".to_string(),
        }
    }
}

impl CalculateMark {
    pub fn from_string(s: String) -> Self {
        match s.as_str() {
            "+" => Self::Plus,
            "-" => Self::Minus,
            "*" => Self::Multiply,
            "/" => Self::Divide,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionalMark {
    None,
    Equal,
    NotEqual,
    Large,
    Small,
    LargeOrEqual,
    SmallOrEqual,
    And,
    Or,
}

impl ToString for ConditionalMark {
    fn to_string(&self) -> String {
        match self {
            ConditionalMark::None => "none".to_string(),
            ConditionalMark::Equal => "==".to_string(),
            ConditionalMark::NotEqual => "!=".to_string(),
            ConditionalMark::Large => ">".to_string(),
            ConditionalMark::Small => "<".to_string(),
            ConditionalMark::LargeOrEqual => ">=".to_string(),
            ConditionalMark::SmallOrEqual => "<=".to_string(),
            ConditionalMark::And => "&&".to_string(),
            ConditionalMark::Or => "||".to_string(),
        }
    }
}

impl ConditionalMark {
    pub fn from_string(s: String) -> Self {
        match s.as_str() {
            "==" => Self::Equal,
            "!=" => Self::NotEqual,
            ">" => Self::Large,
            "<" => Self::Small,
            ">=" => Self::LargeOrEqual,
            "<=" => Self::SmallOrEqual,
            "&&" => Self::And,
            "||" => Self::Or,
            _ => Self::None,
        }
    }
}
