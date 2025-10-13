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

    #[error("scope validation error.")]
    ScopeNotFound,

    #[error("cannot compare different data type: `{a}` and `{b}`.")]
    CompareDiffType { a: String, b: String },

    #[error("variable `{name}` not found.")]
    VariableNotFound { name: String },

    #[error("function `{name}` not found.")]
    FunctionNotFound { name: String },

    #[error("`{name}` pointer data not found")]
    PoniterDataNotFound { name: String },

    #[error("cannot use `{value_type}` in conditional statement.")]
    IllegalTypeInConditional { value_type: String },

    #[error("cannot get `{index_type}` type index from `{value_type}` data.")]
    IllegalIndexType {
        index_type: String,
        value_type: String,
    },

    #[error("cannot find index `{index}` in `{value}` value.")]
    IndexNotFound { index: String, value: String },

    #[error("need arguments number `{need}`, provided `{provided}`.")]
    IllegalArgumentsNumber { need: i16, provided: i16 },

    #[error("you must use a variable to receive anonymous function.")]
    AnonymousFunctionInRoot,

    #[error("cannot find bind function location: `{func}`.")]
    BindFunctionNotFound { func: String },

    #[error("unknown attribute `{attr}` for `{value}` data.")]
    UnknownAttribute { attr: String, value: String },

    #[error("module: `{module}` not found.")]
    ModuleNotFound { module: String },

    #[error("cannot find namespace `{part}` in `{module}` module.")]
    ModulePartNotFound { part: String, module: String },
}
