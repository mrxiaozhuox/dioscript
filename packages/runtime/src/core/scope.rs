use std::collections::HashMap;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Scope {
    pub isolate: bool,
    pub data: HashMap<String, Uuid>,
}

impl Scope {
    pub fn gen() -> Self {
        Self {
            isolate: false,
            data: HashMap::new(),
        }
    }

    pub fn fun() -> Self {
        Self {
            isolate: true,
            data: HashMap::new(),
        }
    }
}
