use std::collections::HashMap;

use dioscript_parser::ast::{CalculateMark, FunctionDefine};
use uuid::Uuid;

use crate::error::RuntimeError;

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
    Function(FunctionType),
    Reference(Uuid),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Rusty((crate::module::RustyFunction, i32)),
    DScript(FunctionDefine),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::None => "none".to_string(),
            Value::String(v) => v.clone(),
            Value::Number(v) => v.to_string(),
            Value::Boolean(v) => v.to_string(),
            Value::List(_) => "[ /* list */ ]".to_string(),
            Value::Dict(_) => "{ /* dict */ }".to_string(),
            Value::Tuple(_) => "( /* tuple */ )".to_string(),
            Value::Element(_) => "element { /* element attributes */  }".to_string(),
            Value::Function(_) => "fn () { /* function impl */  }".to_string(),
            Value::Reference(_) => "/* &reference */".to_string(),
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

    pub fn as_element(&self) -> Option<Element> {
        if let Self::Element(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn to_boolean_data(&self) -> bool {
        match self {
            Value::Number(v) => *v != 0.0,
            Value::Boolean(v) => *v,
            _ => false,
        }
    }

    pub fn calc(&self, o: &Value, s: CalculateMark) -> Result<Value, RuntimeError> {
        if self.value_name() != o.value_name() {
            return Err(RuntimeError::CompareDiffType {
                a: self.value_name(),
                b: o.value_name(),
            });
        }

        match s {
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
                Value::String(v) => Ok(Value::Boolean(*v == o.as_string().unwrap())),
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
                Value::String(v) => Ok(Value::Boolean(*v != o.as_string().unwrap())),
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
        }
    }
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
