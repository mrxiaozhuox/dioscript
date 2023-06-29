use nom::error::ErrorKind;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("[ParseFailed] parser match failed - {kind:?} : {text}")]
    ParseFailure { kind: ErrorKind, text: String },
    #[error("[ParseFailed] have unmatch content: `{content}`")]
    UnMatchContent { content: String },
}