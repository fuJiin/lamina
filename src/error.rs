use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaminaError {
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Parser error: {0}")]
    Parser(String),
    #[error("Lexer error: {0}")]
    #[allow(dead_code)]
    Lexer(String),
    #[error("Evaluation error: {0}")]
    #[allow(dead_code)]
    Evaluation(String),
}

impl From<String> for LaminaError {
    fn from(s: String) -> Self {
        LaminaError::Runtime(s)
    }
}
