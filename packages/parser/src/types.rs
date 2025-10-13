use std::collections::HashMap;

use crate::{
    ast::{FunctionCall, FunctionDefine},
    element::AstElement,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AstValue {
    None,
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<AstValue>),
    Dict(HashMap<String, AstValue>),
    Tuple((Box<AstValue>, Box<AstValue>)),
    Element(AstElement),
    Variable(String),
    VariableIndex((String, Box<AstValue>)),
    FunctionCaller(FunctionCall),
    FunctionDefine(FunctionDefine),
}

impl AstValue {
    pub fn value_name(&self) -> String {
        match self {
            AstValue::None => "none",
            AstValue::String(_) => "string",
            AstValue::Number(_) => "number",
            AstValue::Boolean(_) => "boolean",
            AstValue::List(_) => "list",
            AstValue::Dict(_) => "dict",
            AstValue::Tuple(_) => "tuple",
            AstValue::Element(_) => "element",
            AstValue::Variable(_) => "variable",
            AstValue::VariableIndex(_) => "variable[index]",
            AstValue::FunctionCaller(_) => "call[func]",
            AstValue::FunctionDefine(_) => "def[func]",
        }
        .to_string()
    }

    pub fn as_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn as_string(&self) -> Option<String> {
        if let Self::String(s) = self {
            Some(s.to_string())
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let Self::Boolean(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<Vec<AstValue>> {
        if let Self::List(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_dict(&self) -> Option<HashMap<String, AstValue>> {
        if let Self::Dict(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_tuple(&self) -> Option<(Box<AstValue>, Box<AstValue>)> {
        if let Self::Tuple(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_element(&self) -> Option<AstElement> {
        if let Self::Element(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn as_variable(&self) -> Option<String> {
        if let Self::Variable(s) = self {
            Some(s.to_string())
        } else {
            None
        }
    }
}
