#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("[ParseFailed] parser match failed - {text}")]
    ParseFailure { text: String },
    #[error("[ParseFailed] have unmatch content: `{content}`")]
    UnMatchContent { content: String },
}

