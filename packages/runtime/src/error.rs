use dioscript_parser::error::ParseError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("runtime execute failed: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("parse code failed: {0}")]
    Parse(#[from] ParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    #[error("cannot use `{operator}` operator to `{value_type}` type data.")]
    IllegalOperatorForType {
        operator: String,
        value_type: String,
    },

    #[error("cannot compare different data type: `{a}` and `{b}`.")]
    CompareDiffType { a: String, b: String },

    #[error("variable `{name}` not found.")]
    VariableNotFound { name: String },

    #[error("function `{name}` not found.")]
    FunctionNotFound { name: String },

    #[error("scope node id have some problem.")]
    ScopeNodeIdProblem(#[from] id_tree::NodeIdError),

    #[error("cannot use `{value_type}` in conditional statement.")]
    IllegalTypeInConditional { value_type: String },

    #[error("cannot get `{index_type}` type index from `{value_type}` data.")]
    IllegalIndexType {
        index_type: String,
        value_type: String,
    },

    #[error("cannot find `{index}` in `{value}` value.")]
    IndexNotFound { index: String, value: String },

    #[error("need arguments number `{need}`, provided `{provided}`.")]
    IllegalArgumentsNumber { need: i16, provided: i16 },

    #[error("you must use a variable to receive anonymous function.")]
    AnonymousFunctionInRoot,

    #[error("cannot find bind function location: `{func}`.")]
    BindFunctionNotFound { func: String },

    #[error("cannot find reference: {reference:?}.")]
    ReferenceNotFound { reference: id_tree::NodeId },

    #[error("unknown attribute `{attr}` for `{value}` data.")]
    UnknownAttribute { attr: String, value: String },
}
