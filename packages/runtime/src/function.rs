use dioscript_parser::types::Value;

pub type Function = Box<dyn Fn(Vec<Value>) -> Value>;
