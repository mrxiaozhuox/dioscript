use dioscript_parser::types::Value;

use crate::Runtime;

pub type Function = fn(&mut Runtime, Vec<Value>) -> Value;
