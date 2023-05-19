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

    #[error("cannot compare different data type: `{a}` and `{b}`.")]
    CompareDiffType { a: String, b: String },

    #[error("reference `{name}` not found.")]
    ReferenceNotFound { name: String },

    #[error("scope node id have some problem.")]
    ScopeNodeIdProblem(#[from] id_tree::NodeIdError),

    #[error("cannot use `{value_type}` in conditional statement.")]
    IllegalTypeInConditional { value_type: String },
}
