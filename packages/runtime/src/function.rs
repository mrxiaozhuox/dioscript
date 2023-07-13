use std::collections::HashMap;

use crate::{types::Value, Runtime};

pub type Function = fn(&mut Runtime, Vec<Value>) -> Value;

#[derive(Debug)]
pub struct MethodBinder {
    pub target: BindTarget,
    pub functions: HashMap<String, (Function, i32)>,
}

impl MethodBinder {
    pub fn new(target: BindTarget) -> Self {
        Self {
            target,
            functions: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, func: (Function, i32)) {
        self.functions.insert(name.to_string(), func);
    }

    pub fn collect(&self) -> (BindTarget, HashMap<String, Value>) {
        let mut result = HashMap::new();
        for (name, info) in &self.functions {
            result.insert(
                name.clone(),
                Value::Function(crate::types::FunctionType::BindFunction(info.clone())),
            );
        }
        (self.target.clone(), result)
    }
}

#[derive(Debug, Clone)]
pub enum BindTarget {
    Root,

    String,
    Number,
    Boolean,
    Tuple,
    List,
    Dict,
    Element,
    Function,

    Struct(String),
}

impl BindTarget {
    pub fn to_string(&self) -> String {
        match self {
            BindTarget::Root => "".to_string(),
            BindTarget::String => "string".to_string(),
            BindTarget::Number => "number".to_string(),
            BindTarget::Boolean => "boolean".to_string(),
            BindTarget::Tuple => "tuple".to_string(),
            BindTarget::List => "list".to_string(),
            BindTarget::Dict => "dict".to_string(),
            BindTarget::Element => "element".to_string(),
            BindTarget::Function => "function".to_string(),
            BindTarget::Struct(s) => s.to_lowercase(),
        }
    }
}
