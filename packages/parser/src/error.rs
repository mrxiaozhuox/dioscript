use nom::error::ErrorKind;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("runtime execute failed.")]
    Runtime(#[from] RuntimeError),
    #[error("parse code failed.")]
    Parse(#[from] ParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("[ParseFailed] parser match failed - {kind:?} : {text}")]
    ParseFailure { kind: ErrorKind, text: String },
    #[error("[ParseFailed] have unmatch content: `{content}`")]
    UnMatchContent { content: String },
}

#[derive(ThisError, Debug)]
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

    #[error("you are trying to call meta bind function.")]
    CallMeatBindFunction,

    #[error("cannot find bind function location: `{func}`.")]
    BindFunctionNotFound { func: String },
}
