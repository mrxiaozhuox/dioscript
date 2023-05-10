use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub content: String,
    pub children: Vec<Element>,
}
