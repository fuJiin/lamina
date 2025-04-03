
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaminaError {
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    #[error("Parser error: {0}")]
    ParserError(String),
}

impl From<String> for LaminaError {
    fn from(s: String) -> Self {
        LaminaError::RuntimeError(s)
    }
}
