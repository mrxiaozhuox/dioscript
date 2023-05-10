use std::collections::HashMap;

use crate::types::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: HashMap<String, Value>,
    pub content: String,
    pub children: Vec<Element>,
}
