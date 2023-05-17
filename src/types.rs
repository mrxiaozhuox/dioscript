use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
    Tuple((Box<Value>, Box<Value>)),
    Element(crate::element::Element),
    Reference(String),
}

impl Value {
    pub fn value_name(&self) -> String {
        match self {
            Value::None => "none",
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Boolean(_) => "boolean",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Tuple(_) => "tuple",
            Value::Element(_) => "element",
            Value::Reference(_) => "reference",
        }
        .to_string()
    }

    pub fn as_none(&self) -> bool {
        if let Self::None = self {
            true
        } else {
            false
        }
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

    pub fn as_list(&self) -> Option<Vec<Value>> {
        if let Self::List(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_dict(&self) -> Option<HashMap<String, Value>> {
        if let Self::Dict(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_tuple(&self) -> Option<(Box<Value>, Box<Value>)> {
        if let Self::Tuple(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_element(&self) -> Option<crate::element::Element> {
        if let Self::Element(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn as_reference(&self) -> Option<String> {
        if let Self::Reference(s) = self {
            Some(s.to_string())
        } else {
            None
        }
    }
}
