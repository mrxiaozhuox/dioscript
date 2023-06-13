use std::collections::HashMap;

use crate::{
    ast::{CalculateMark, FunctionCall, FunctionDefine},
    element::{AstElement, Element},
    error::RuntimeError,
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
    Function(FunctionDefine),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::None => "none".to_string(),
            Value::String(v) => v.clone(),
            Value::Number(v) => v.to_string(),
            Value::Boolean(v) => v.to_string(),
            Value::List(_) => "[ list ]".to_string(),
            Value::Dict(_) => "{ dict }".to_string(),
            Value::Tuple(_) => "( tuple )".to_string(),
            Value::Element(_) => "{ element }".to_string(),
            Value::Function(_) => "[ function ]".to_string(),
        }
    }
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
            Value::Function(_) => "function",
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

    pub fn as_element(&self) -> Option<Element> {
        if let Self::Element(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn to_boolean_data(&self) -> bool {
        match self {
            Value::None => false,
            Value::String(_) => false,
            Value::Number(v) => *v != 0.0,
            Value::Boolean(v) => *v,
            Value::List(_) => false,
            Value::Dict(_) => false,
            Value::Tuple(v) => {
                let a = v.0.clone();
                let b = v.1.clone();
                a.to_boolean_data() && b.to_boolean_data()
            }
            Value::Element(_) => false,
            Value::Function(_) => false,
        }
    }

    pub fn calc(&self, o: &Value, s: CalculateMark) -> Result<Value, RuntimeError> {
        if self.value_name() != o.value_name() {
            return Err(RuntimeError::CompareDiffType {
                a: self.value_name(),
                b: o.value_name(),
            });
        }

        return match s {
            CalculateMark::Plus => match self {
                Value::String(v) => Ok(Self::String(format!("{}{}", v, o.as_string().unwrap()))),
                Value::Number(v) => Ok(Self::Number(v + o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "+".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Minus => match self {
                Value::Number(v) => Ok(Self::Number(v - o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "+".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Multiply => match self {
                Value::Number(v) => Ok(Self::Number(v * o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "+".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Divide => match self {
                Value::Number(v) => Ok(Self::Number(v / o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "+".to_string(),
                    value_type: self.value_name(),
                }),
            },

            CalculateMark::Equal => match self {
                Value::String(v) => Ok(Value::Boolean(v.to_string() == o.as_string().unwrap())),
                Value::Number(v) => Ok(Value::Boolean(*v == o.as_number().unwrap())),
                Value::Boolean(v) => Ok(Value::Boolean(*v == o.as_boolean().unwrap())),
                Value::List(v) => Ok(Value::Boolean(v.clone() == o.as_list().unwrap())),
                Value::Dict(v) => Ok(Value::Boolean(v.clone() == o.as_dict().unwrap())),
                Value::Tuple(v) => Ok(Value::Boolean(v.clone() == o.as_tuple().unwrap())),
                Value::Element(v) => Ok(Value::Boolean(v.clone() == o.as_element().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "==".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::NotEqual => match self {
                Value::String(v) => Ok(Value::Boolean(v.to_string() != o.as_string().unwrap())),
                Value::Number(v) => Ok(Value::Boolean(*v != o.as_number().unwrap())),
                Value::Boolean(v) => Ok(Value::Boolean(*v != o.as_boolean().unwrap())),
                Value::List(v) => Ok(Value::Boolean(v.clone() != o.as_list().unwrap())),
                Value::Dict(v) => Ok(Value::Boolean(v.clone() != o.as_dict().unwrap())),
                Value::Tuple(v) => Ok(Value::Boolean(v.clone() != o.as_tuple().unwrap())),
                Value::Element(v) => Ok(Value::Boolean(v.clone() != o.as_element().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "!=".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Large => match self {
                Value::Number(v) => Ok(Value::Boolean(*v > o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: ">".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Small => match self {
                Value::Number(v) => Ok(Value::Boolean(*v < o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "<".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::LargeOrEqual => match self {
                Value::Number(v) => Ok(Value::Boolean(*v >= o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: ">=".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::SmallOrEqual => match self {
                Value::Number(v) => Ok(Value::Boolean(*v <= o.as_number().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "<=".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::And => match self {
                Value::Boolean(v) => Ok(Value::Boolean(*v && o.as_boolean().unwrap())),
                _ => Err(RuntimeError::IllegalOperatorForType {
                    operator: "&&".to_string(),
                    value_type: self.value_name(),
                }),
            },
            CalculateMark::Or => match self {
                Value::Boolean(v) => Ok(Value::Boolean(*v || o.as_boolean().unwrap())),
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
