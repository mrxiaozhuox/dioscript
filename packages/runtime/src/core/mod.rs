pub mod error;
pub mod module;
pub mod names;
pub mod runtime;
pub mod scope;
pub mod types;

pub mod io;

#[derive(Debug, Clone)]
pub enum DataType {
    Variable(types::Value),
}

impl DataType {
    pub fn as_variable(&self) -> Option<types::Value> {
        #[allow(unreachable_patterns)]
        #[allow(irrefutable_let_patterns)]
        if let Self::Variable(r) = self {
            return Some(r.clone());
        }
        None
    }
}
