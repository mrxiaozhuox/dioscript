use std::collections::HashMap;

use crate::{parser::CalcExpr, types::AstValue};

#[derive(Debug, Clone, PartialEq)]
pub struct AstElement {
    pub name: String,
    pub attributes: HashMap<String, AstValue>,
    pub content: Vec<AstElementContentType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstElementContentType {
    Children(AstElement),
    Content(String),
    Condition(crate::ast::ConditionalStatement),
    Loop(crate::ast::LoopStatement),
    InlineExpr(CalcExpr),
}
