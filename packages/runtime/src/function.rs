use std::collections::HashMap;

use dioscript_parser::{error::RuntimeError, types::Value};

use crate::Runtime;

pub type Function = fn(&mut Runtime, Vec<Value>) -> Value;

pub struct Exporter {
    sign: String,
    functions: HashMap<String, (Function, i32)>,
}

impl Exporter {
    pub fn new(sign: &str) -> Self {
        Self {
            functions: HashMap::new(),
            sign: sign.to_string(),
        }
    }
    pub fn insert(&mut self, n: &str, f: (Function, i32)) {
        self.functions.insert(n.to_string(), f);
    }
    pub fn bind(&self, rt: &mut Runtime) -> Result<Value, RuntimeError> {
        let mut map = HashMap::new();
        for (n, f) in &self.functions {
            let (_, v) = rt.add_bind_function(&self.sign, n, f.clone())?;
            map.insert(n.to_string(), v);
        }
        Ok(Value::Dict(map))
    }
}
