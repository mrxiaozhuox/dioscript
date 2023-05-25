use std::collections::HashMap;

use crate::types::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: HashMap<String, Value>,
    pub content: Vec<ElementContentType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementContentType {
    Children(Element),
    Content(String),
    Condition(crate::ast::ConditionalStatement),
    Variable(String),
}
