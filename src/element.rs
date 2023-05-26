use std::collections::HashMap;

use crate::types::{AstValue, Value};

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
    pub attributes: HashMap<String, Value>,
    pub content: Vec<ElementContentType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementContentType {
    Children(Element),
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

        for (k, v) in from.attributes {
            this.attributes.insert(k, Value::from_ast_value(v));
        }

        for i in from.content {
            match i {
                AstElementContentType::Children(v) => {
                    this.content
                        .push(ElementContentType::Children(Element::from_ast_element(v)));
                }
                AstElementContentType::Content(v) => {
                    this.content.push(ElementContentType::Content(v));
                }
                _ => {}
            }
        }

        this
    }

    pub fn to_html(&self) -> String {
        let mut attr_str = String::new();
        for (name, value) in &self.attributes {
            if let Value::String(value) = value {
                attr_str.push_str(&format!(" {0}=\"{1}\"", name, value));
            } else if let Value::Boolean(value) = value {
                if *value {
                    attr_str.push_str(&format!(" {name}"));
                }
            } else if let Value::Number(value) = value {
                attr_str.push_str(&format!(" {0}=\"{1}\"", name, value));
            }
        }
        let mut content_str = String::new();
        for sub in &self.content {
            let v = match sub {
                ElementContentType::Children(v) => v.to_html(),
                ElementContentType::Content(v) => v.clone(),
            };
            content_str.push_str(&v);
        }
        let result = format!("<{tag}{attr_str}>{content_str}</{tag}>", tag = self.name);
        result
    }
}
