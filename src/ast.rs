use std::str::FromStr;

use crate::types::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct DioscriptAst {
    pub stats: Vec<DioAstStatement>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpr(pub Vec<(ConditionalSignal, (bool, Value))>);

#[derive(Debug, Clone, PartialEq)]
pub enum SubExpr {
    Single((bool, Value)),
    Pair(ConditionalExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionalSignal {
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

impl ToString for ConditionalSignal {
    fn to_string(&self) -> String {
        match self {
            ConditionalSignal::None => "".to_string(),
            ConditionalSignal::Equal => "==".to_string(),
            ConditionalSignal::NotEqual => "!=".to_string(),
            ConditionalSignal::Large => ">".to_string(),
            ConditionalSignal::Small => "<".to_string(),
            ConditionalSignal::LargeOrEqual => ">=".to_string(),
            ConditionalSignal::SmallOrEqual => "<=".to_string(),
            ConditionalSignal::And => "&&".to_string(),
            ConditionalSignal::Or => "||".to_string(),
        }
    }
}

impl ConditionalSignal {
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
