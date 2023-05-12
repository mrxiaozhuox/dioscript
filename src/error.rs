use thiserror::Error as Terr;

#[derive(Terr, Debug)]
pub enum Error {
    #[error("runtime execute failed.")]
    Runtime(#[from] RuntimeError),
}

#[derive(Terr, Debug)]
pub enum RuntimeError {
    #[error("cannot use `{operator}` operator to `{value_type}` type data.")]
    IllegalOperatorForType {
        operator: String,
        value_type: String,
    },
}

impl RuntimeError {
    pub fn illegal_operator_for_type(operator: &str, value_type: &str) -> Self {
        Self::IllegalOperatorForType {
            operator: operator.to_string(),
            value_type: value_type.to_string(),
        }
    }
}
