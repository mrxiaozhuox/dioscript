use std::fmt::Display;

use nom::{combinator::all_consuming, Finish};

use crate::{
    error::{simplify_error, ParseError},
    parser::{parse_rsx, CalcExpr},
};

#[derive(Debug, Clone, PartialEq)]
pub struct DioscriptAst {
    pub stats: Vec<DioAstStatement>,
}

impl DioscriptAst {
    pub fn from_string(message: &str) -> Result<Self, ParseError> {
        let v = all_consuming(parse_rsx)(message).finish();
        if let Ok((text, ast)) = v {
            if text.trim().is_empty() {
                Ok(DioscriptAst { stats: ast })
            } else {
                let content = text.lines().next().unwrap_or("");
                Err(ParseError::UnMatchContent {
                    content: content.to_string(),
                })
            }
        } else {
            let err = v.err().unwrap();

            let err = simplify_error(message, err);
            Err(ParseError::ParseFailure { text: err })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DioAstStatement {
    VariableAss(VariableDefine),
    ReturnValue(CalcExpr),
    IfStatement(ConditionalStatement),
    LoopStatement(LoopStatement),
    LineComment(String),
    FunctionCall(FunctionCall),
    FunctionDefine(FunctionDefine),

    CalcExpr(CalcExpr),

    ModuleUse(UseStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDefine {
    pub new: bool,
    pub name: String,
    pub expr: CalcExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: FunctionName,
    pub arguments: Vec<CalcExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionName {
    Single(String),
    Namespace(Vec<String>),
}

impl Display for FunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            FunctionName::Single(s) => s.to_string(),
            FunctionName::Namespace(n) => n.join("::"),
        };
        write!(f, "{}", res)
    }
}

impl FunctionName {
    pub fn as_single(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefine {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub variadic_param: Option<String>,
    pub inner: Vec<DioAstStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalStatement {
    pub condition: CalcExpr,
    pub inner: Vec<DioAstStatement>,
    pub otherwise: Option<Vec<DioAstStatement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopStatement {
    pub execute_type: LoopExecuteType,
    pub inner: Vec<DioAstStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement(pub Vec<String>);

#[derive(Debug, Clone, PartialEq)]
pub enum LoopExecuteType {
    Conditional(CalcExpr),
    Iter { iter: CalcExpr, var: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalculateMark {
    None,

    Plus,
    Minus,
    Multiply,
    Divide,
    Mod,

    Equal,
    NotEqual,
    Large,
    Small,
    LargeOrEqual,
    SmallOrEqual,
    And,
    Or,
}

impl Display for CalculateMark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            CalculateMark::None => "none",
            CalculateMark::Plus => "+",
            CalculateMark::Minus => "-",
            CalculateMark::Multiply => "*",
            CalculateMark::Divide => "/",
            CalculateMark::Mod => "%",
            CalculateMark::Equal => "==",
            CalculateMark::NotEqual => "!=",
            CalculateMark::Large => ">",
            CalculateMark::Small => "<",
            CalculateMark::LargeOrEqual => ">=",
            CalculateMark::SmallOrEqual => "<=",
            CalculateMark::And => "&&",
            CalculateMark::Or => "||",
        };
        write!(f, "{}", res)
    }
}

impl CalculateMark {
    pub fn from_string(s: String) -> Self {
        match s.as_str() {
            "+" => Self::Plus,
            "-" => Self::Minus,
            "*" => Self::Multiply,
            "/" => Self::Divide,
            "%" => Self::Mod,

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
