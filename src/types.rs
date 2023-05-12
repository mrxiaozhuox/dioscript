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
}
