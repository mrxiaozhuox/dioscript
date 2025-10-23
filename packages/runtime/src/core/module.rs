use std::collections::HashMap;

use dioscript_parser::ast::DioscriptAst;
use uuid::Uuid;

use super::{
    error::RuntimeError,
    runtime::Runtime,
    types::{FunctionType, Value},
};

pub struct RustyExecutor<'rt> {
    pub runtime: &'rt mut Runtime,
}

impl<'rt> RustyExecutor<'rt> {
    /// generate a RustyExecutor and bind Runtime
    pub fn bind(runtime: &'rt mut Runtime) -> Self {
        Self { runtime }
    }

    /// execute code in a new
    pub fn execute(&mut self, code: &str, isolate: bool) -> Result<Value, RuntimeError> {
        let ast = DioscriptAst::from_string(code)
            .map_err(|e| RuntimeError::DynamicParseFailed { err: e.to_string() })?;
        if isolate {
            self.runtime.execute_isolate_scope(ast.stats)
        } else {
            self.runtime.execute_scope(ast.stats)
        }
    }

    /// set a variable (in dioscript runtime)
    /// INFO: this variable will only work on current scope (function calling scope)
    ///
    /// ```rust
    /// fn rusty_function_ex(rt: RustyExecutor, args: Vec<Value>) -> Result<Value, RuntimeError> {
    ///     rt.set_var("count", Value::Number(0));
    /// }
    /// ```
    /// should be equal with:
    /// ```dioscript
    /// fn rusty_function_ex() {
    ///     let count = 0;
    /// }
    /// ```
    pub fn set_var(&mut self, name: &str, value: Value) -> Result<Uuid, RuntimeError> {
        if self.runtime.get_var(name).is_err() {
            self.runtime.create_var(name, value)
        } else {
            self.runtime.set_var(name, value)
        }
    }

    /// get a variable from dioscript runtime
    ///
    /// ```rust
    /// fn rusty_function_ex(rt: RustyExecutor, args: Vec<Value>) -> Result<Value, RuntimeError> {
    ///     rt.get_var("count");
    /// }
    /// ```
    /// should be equal with:
    /// ```dioscript
    /// fn rusty_function_ex() {
    ///     // read this variable
    ///     count;
    /// }
    /// ```
    pub fn get_var(&mut self, name: &str) -> Result<(Uuid, Value), RuntimeError> {
        self.runtime.get_var(name)
    }

    /// get value from reference (deref)
    pub fn get_reference(&mut self, id: &Uuid) -> Result<Value, RuntimeError> {
        self.runtime.get_ref_value(id)
    }

    /// update reference value
    pub fn set_reference(&mut self, id: &Uuid, value: Value) -> Result<(), RuntimeError> {
        self.runtime.set_ref_value(id, value)
    }

    /// send content to output handler
    pub fn output(&mut self, output_type: OutputType, content: Value) {
        match output_type {
            OutputType::Print => {
                self.runtime.output_handler.print(content);
            }
        }
    }
}

#[derive(Debug)]
pub enum OutputType {
    Print,
}

pub type RustyFunction = fn(RustyExecutor, Vec<Value>) -> Result<Value, RuntimeError>;

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

impl Default for ModuleGenerator {
    fn default() -> Self {
        Self::new()
    }
}
