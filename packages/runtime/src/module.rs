use std::collections::HashMap;


use crate::{types::{Value, FunctionType}, Runtime};

pub type RustyFunction = fn(&mut Runtime, Vec<Value>) -> Value;

#[derive(Clone)]
pub enum ModuleItem {
    Function(FunctionType),
    Variable(Value),
    SubModule(ModuleInfo),
}


#[derive(Clone)]
pub struct ModuleInfo(pub HashMap<String, ModuleItem>);

pub struct ModuleGenerator(pub HashMap<String, ModuleItem>);
impl ModuleGenerator {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn insert(&mut self, k: &str, v: ModuleItem) {
        self.0.insert(k.to_string(), v);
    }

    pub fn insert_rusty_function(&mut self, k: &str, func: RustyFunction, arg: i32) {
        self.insert(k, ModuleItem::Function(FunctionType::Rusty((func, arg))))
    }


    pub fn insert_sub_module(&mut self, k: &str, v: ModuleGenerator) {
        self.insert(k, ModuleItem::SubModule(ModuleInfo(v.0)));
    }

    pub fn to_module_item(self) -> ModuleItem {
        ModuleItem::SubModule(ModuleInfo(self.0))
    }

}
