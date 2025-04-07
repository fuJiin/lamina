use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
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
    #[error("Compilation error: {0}")]
    #[allow(dead_code)]
    Compilation(String),
    #[error("IO error: {0}")]
    #[allow(dead_code)]
    IO(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Runtime(s)
    }
}
