mod core;
mod library;

use core::error::RuntimeError;
use core::io::output::ConsoleOutputHandler;
use core::runtime::Runtime;

// type export
pub use core::types::Element;
pub use core::types::ElementContentType;
pub use core::types::FunctionType;
pub use core::types::Value;

// module export
pub use core::io::output::OutputHandler;
pub use core::module::ModuleGenerator;
pub use core::module::RustyExecutor;

use std::collections::HashMap;

use dioscript_parser::ast::DioscriptAst;
use uuid::Uuid;

use crate::core::scope::Scope;

pub struct Executor {
    pub runtime: Runtime,
}

impl Executor {
    pub fn init() -> Self {
        let mut runtime = Runtime {
            scopes: vec![],
            data: HashMap::new(),
            modules: library::built_in().0,
            namespace_use: Default::default(),
            output_handler: Box::new(ConsoleOutputHandler),
        };

        runtime.enter_scope(false);

        Self { runtime }
    }

    pub fn with_output_handler(&mut self, handler: Box<dyn OutputHandler>) {
        self.runtime.output_handler = handler;
    }

    pub fn execute(&mut self, ast: DioscriptAst) -> Result<Value, RuntimeError> {
        self.runtime.execute_scope_without_new_scope(ast.stats)
    }

    pub fn debug_scopes_info(&self) -> Vec<Scope> {
        self.runtime.scopes.clone()
    }

    pub fn debug_data_info(&self) -> HashMap<Uuid, Value> {
        self.runtime
            .data
            .iter()
            .filter_map(|(id, v)| {
                v.as_variable().map(|v| (*id, v))
                // 123
            })
            .collect::<HashMap<_, _>>()
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.runtime.leave_scope();
    }
}
