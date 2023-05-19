use std::collections::HashMap;

use crate::{ast::ConditionalSignal, error::RuntimeError};

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

    pub fn compare(&self, o: &Value, s: ConditionalSignal) -> Result<bool, RuntimeError> {
        if self.value_name() != o.value_name() {
            return Err(RuntimeError::CompareDiffType {
                a: self.value_name(),
                b: o.value_name(),
            });
        }

        return match s {
            ConditionalSignal::Equal => match self {
                Value::String(v) => Ok(v.to_string() == o.as_string().unwrap()),
                Value::Number(v) => Ok(*v == o.as_number().unwrap()),
                Value::Boolean(v) => Ok(*v == o.as_boolean().unwrap()),
                Value::List(v) => Ok(v.clone() == o.as_list().unwrap()),
                Value::Dict(v) => Ok(v.clone() == o.as_dict().unwrap()),
                Value::Tuple(v) => Ok(v.clone() == o.as_tuple().unwrap()),
                Value::Element(v) => Ok(v.clone() == o.as_element().unwrap()),
                Value::Reference(v) => Ok(v.to_string() == o.as_string().unwrap()),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "==".to_string(),
                    value_type: self.value_name(),
                }),
            },
            ConditionalSignal::NotEqual => match self {
                Value::String(v) => Ok(v.to_string() != o.as_string().unwrap()),
                Value::Number(v) => Ok(*v != o.as_number().unwrap()),
                Value::Boolean(v) => Ok(*v != o.as_boolean().unwrap()),
                Value::List(v) => Ok(v.clone() != o.as_list().unwrap()),
                Value::Dict(v) => Ok(v.clone() != o.as_dict().unwrap()),
                Value::Tuple(v) => Ok(v.clone() != o.as_tuple().unwrap()),
                Value::Element(v) => Ok(v.clone() != o.as_element().unwrap()),
                Value::Reference(v) => Ok(v.to_string() != o.as_string().unwrap()),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "!=".to_string(),
                    value_type: self.value_name(),
                }),
            },
            ConditionalSignal::Large => match self {
                Value::Number(v) => Ok(*v > o.as_number().unwrap()),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: ">".to_string(),
                    value_type: self.value_name(),
                }),
            },
            ConditionalSignal::Small => match self {
                Value::Number(v) => Ok(*v < o.as_number().unwrap()),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "<".to_string(),
                    value_type: self.value_name(),
                }),
            },
            ConditionalSignal::LargeOrEqual => match self {
                Value::Number(v) => Ok(*v >= o.as_number().unwrap()),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: ">=".to_string(),
                    value_type: self.value_name(),
                }),
            },
            ConditionalSignal::SmallOrEqual => match self {
                Value::Number(v) => Ok(*v <= o.as_number().unwrap()),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "<=".to_string(),
                    value_type: self.value_name(),
                }),
            },
            ConditionalSignal::And => match self {
                Value::Boolean(v) => Ok(*v && o.as_boolean().unwrap()),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "&&".to_string(),
                    value_type: self.value_name(),
                }),
            },
            ConditionalSignal::Or => match self {
                Value::Boolean(v) => Ok(*v || o.as_boolean().unwrap()),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "||".to_string(),
                    value_type: self.value_name(),
                }),
            },
            _ => Err(RuntimeError::IllegalOperatorForType {
                operator: "None".to_string(),
                value_type: o.value_name(),
            }),
        };
    }
}
