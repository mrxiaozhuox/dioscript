use std::collections::HashMap;

use crate::{ast::CalculateMark, error::RuntimeError, element::{AstElement, Element}};

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

    pub fn calc(&self, o: &AstValue, s: CalculateMark) -> Result<AstValue, RuntimeError> {
        if self.value_name() != o.value_name() {
            return Err(RuntimeError::CompareDiffType {
                a: self.value_name(),
                b: o.value_name(),
            });
        }

        return match s {
            CalculateMark::Plus => match self {
                AstValue::String(v) => Ok(Self::String(format!("{}{}", v, o.as_string().unwrap()))),
                AstValue::Number(v) => Ok(Self::Number(v + o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "+".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Minus => match self {
                AstValue::Number(v) => Ok(Self::Number(v - o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "+".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Multiply => match self {
                AstValue::Number(v) => Ok(Self::Number(v * o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "+".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Divide => match self {
                AstValue::Number(v) => Ok(Self::Number(v / o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "+".to_string(),
                    value_type: self.value_name(),
                }),
            },

            CalculateMark::Equal => match self {
                AstValue::String(v) => {
                    Ok(AstValue::Boolean(v.to_string() == o.as_string().unwrap()))
                }
                AstValue::Number(v) => Ok(AstValue::Boolean(*v == o.as_number().unwrap())),
                AstValue::Boolean(v) => Ok(AstValue::Boolean(*v == o.as_boolean().unwrap())),
                AstValue::List(v) => Ok(AstValue::Boolean(v.clone() == o.as_list().unwrap())),
                AstValue::Dict(v) => Ok(AstValue::Boolean(v.clone() == o.as_dict().unwrap())),
                AstValue::Tuple(v) => Ok(AstValue::Boolean(v.clone() == o.as_tuple().unwrap())),
                AstValue::Element(v) => Ok(AstValue::Boolean(v.clone() == o.as_element().unwrap())),
                AstValue::Variable(v) => {
                    Ok(AstValue::Boolean(v.to_string() == o.as_string().unwrap()))
                }
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "==".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::NotEqual => match self {
                AstValue::String(v) => {
                    Ok(AstValue::Boolean(v.to_string() != o.as_string().unwrap()))
                }
                AstValue::Number(v) => Ok(AstValue::Boolean(*v != o.as_number().unwrap())),
                AstValue::Boolean(v) => Ok(AstValue::Boolean(*v != o.as_boolean().unwrap())),
                AstValue::List(v) => Ok(AstValue::Boolean(v.clone() != o.as_list().unwrap())),
                AstValue::Dict(v) => Ok(AstValue::Boolean(v.clone() != o.as_dict().unwrap())),
                AstValue::Tuple(v) => Ok(AstValue::Boolean(v.clone() != o.as_tuple().unwrap())),
                AstValue::Element(v) => Ok(AstValue::Boolean(v.clone() != o.as_element().unwrap())),
                AstValue::Variable(v) => {
                    Ok(AstValue::Boolean(v.to_string() != o.as_string().unwrap()))
                }
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "!=".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Large => match self {
                AstValue::Number(v) => Ok(AstValue::Boolean(*v > o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: ">".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Small => match self {
                AstValue::Number(v) => Ok(AstValue::Boolean(*v < o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "<".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::LargeOrEqual => match self {
                AstValue::Number(v) => Ok(AstValue::Boolean(*v >= o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: ">=".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::SmallOrEqual => match self {
                AstValue::Number(v) => Ok(AstValue::Boolean(*v <= o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "<=".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::And => match self {
                AstValue::Boolean(v) => Ok(AstValue::Boolean(*v && o.as_boolean().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "&&".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Or => match self {
                AstValue::Boolean(v) => Ok(AstValue::Boolean(*v || o.as_boolean().unwrap())),
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

    pub fn to_boolean_data(&self) -> bool {
        match self {
            AstValue::None => false,
            AstValue::String(_) => false,
            AstValue::Number(v) => *v != 0.0,
            AstValue::Boolean(v) => *v,
            AstValue::List(_) => false,
            AstValue::Dict(_) => false,
            AstValue::Tuple(v) => {
                let a = v.0.clone();
                let b = v.1.clone();
                a.to_boolean_data() && b.to_boolean_data()
            }
            AstValue::Element(_) => false,
            AstValue::Variable(_) => false,
            AstValue::VariableIndex(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
    Tuple((Box<Value>, Box<Value>)),
    Element(Element),
}

impl Value {
    pub fn from_ast_value(from: AstValue) -> Self {
        match from {
            AstValue::None => Value::None,
            AstValue::String(v) => Value::String(v),
            AstValue::Number(v) => Value::Number(v),
            AstValue::Boolean(v) => Value::Boolean(v),
            AstValue::List(v) => Value::List(
                v.iter()
                    .map(|i| Value::from_ast_value(i.clone()))
                    .collect::<Vec<Value>>(),
            ),
            AstValue::Dict(v) => {
                let mut r = HashMap::new();
                for (k, d) in v {
                    r.insert(k, Value::from_ast_value(d));
                }
                Value::Dict(r)
            }
            AstValue::Tuple(v) => Value::Tuple((
                Box::new(Value::from_ast_value(*v.0)),
                Box::new(Value::from_ast_value(*v.1)),
            )),
            AstValue::Element(v) => Value::Element(Element::from_ast_element(v)),
            AstValue::Variable(_) => Value::None,
            AstValue::VariableIndex(_) => Value::None,
        }
    }
}
