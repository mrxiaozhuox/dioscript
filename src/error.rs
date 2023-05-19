use nom::error::ErrorKind;
use thiserror::Error as Terr;

#[derive(Terr, Debug)]
pub enum Error {
    #[error("runtime execute failed.")]
    Runtime(#[from] RuntimeError),
    #[error("parse code failed.")]
    Parse(#[from] ParseError),
}

#[derive(Terr, Debug)]
pub enum ParseError {
    #[error("parse failure - [{kind:?}]: {text}")]
    ParseFailure { kind: ErrorKind, text: String },
}

#[derive(Terr, Debug)]
pub enum RuntimeError {
    #[error("cannot use `{operator}` operator to `{value_type}` type data.")]
    IllegalOperatorForType {
        operator: String,
        value_type: String,
    },

    #[error("cannot compare different data type: `{a}` and `{b}`.")]
    CompareDiffType { a: String, b: String },

    #[error("reference `{name}` not found.")]
    ReferenceNotFound { name: String },

    #[error("scope node id have some problem.")]
    ScopeNodeIdProblem(#[from] id_tree::NodeIdError),

    #[error("cannot use `{value_type}` in conditional statement.")]
    IllegalTypeInConditional { value_type: String },
}
