
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaminaError {
    #[error("Lexer error: {0}")]
    LexerError(String),
    
    #[error("Parser error: {0}")]
    ParserError(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}
