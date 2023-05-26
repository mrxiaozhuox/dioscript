use std::collections::HashMap;

use crate::types::AstValue;

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
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: HashMap<String, AstValue>,
    pub content: Vec<ElementContentType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementContentType {
    Children(AstElement),
    Content(String),
}

impl Element {
    pub fn from_ast_element(from: AstElement) -> Self {
        let mut this = Self {
            name: String::new(),
            attributes: HashMap::new(),
            content: vec![],
        };

        this.name = from.name;
        this.attributes = from.attributes;

        for i in from.content {
            match i {
                AstElementContentType::Children(v) => {
                    this.content.push(ElementContentType::Children(v));
                }
                AstElementContentType::Content(v) => {
                    this.content.push(ElementContentType::Content(v));
                }
                _ => {}
            }
        }

        this
    }
}
