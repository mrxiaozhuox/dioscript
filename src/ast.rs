use nom::Finish;

use crate::{
    error::{Error, ParseError},
    parser::{parse_rsx, CalcExpr},
    types::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub struct DioscriptAst {
    pub stats: Vec<DioAstStatement>,
}

impl DioscriptAst {
    pub fn from_string(message: &str) -> Result<Self, ParseError> {
        let v = parse_rsx(message).finish();
        if let Ok(v) = v {
            Ok(DioscriptAst { stats: v.1 })
        } else {
            let err = v.err().unwrap();
            Err(ParseError::ParseFailure {
                kind: err.code,
                text: err.input.to_string(),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DioAstStatement {
    ReferenceAss((String, CalcExpr)),
    ReturnValue(CalcExpr),
    IfStatement(ConditionalStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalStatement {
    pub condition: ConditionalExpr,
    pub inner: Vec<DioAstStatement>,
    pub otherwise: Option<Vec<DioAstStatement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpr(pub Vec<(CalculateMark, SubExpr)>);

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

    Equal,
    NotEqual,
    Large,
    Small,
    LargeOrEqual,
    SmallOrEqual,
    And,
    Or,
}

impl ToString for CalculateMark {
    fn to_string(&self) -> String {
        match self {
            CalculateMark::None => "none".to_string(),

            CalculateMark::Plus => "+".to_string(),
            CalculateMark::Minus => "-".to_string(),
            CalculateMark::Multiply => "*".to_string(),
            CalculateMark::Divide => "/".to_string(),

            CalculateMark::Equal => "==".to_string(),
            CalculateMark::NotEqual => "!=".to_string(),
            CalculateMark::Large => ">".to_string(),
            CalculateMark::Small => "<".to_string(),
            CalculateMark::LargeOrEqual => ">=".to_string(),
            CalculateMark::SmallOrEqual => "<=".to_string(),
            CalculateMark::And => "&&".to_string(),
            CalculateMark::Or => "||".to_string(),
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
