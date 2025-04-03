
use logos::Logos;
use crate::error::LaminaError;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[token("(")]
    LParen,
    
    #[token(")")]
    RParen,
    
    #[token("'")]
    Quote,
    
    #[token("`")]
    Quasiquote,
    
    #[token(",")]
    Unquote,
    
    #[token(",@")]
    UnquoteSplicing,
    
    #[token("#t")]
    True,
    
    #[token("#f")]
    False,
    
    #[regex(r#"#\\[a-zA-Z]+"#, |lex| lex.slice()[2..].chars().next())]
    #[regex(r#"#\\."#, |lex| lex.slice().chars().nth(2))]
    Character(char),
    
    #[regex("[0-9]+(?:/[0-9]+)?", |lex| lex.slice().to_string())]
    #[regex("[0-9]+\\.[0-9]+", |lex| lex.slice().to_string())]
    Number(String),
    
    #[regex(r#""([^"\\]|\\[\\\"nt])*""#, |lex| {
        lex.slice()[1..lex.slice().len()-1].to_string()
    })]
    String(String),
    
    #[regex("[a-zA-Z!$%&*/:<=>?^_~][a-zA-Z0-9!$%&*/:<=>?^_~+-\\.@]*", |lex| lex.slice().to_string())]
    Symbol(String),
    
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[regex(r";[^\n]*\n", logos::skip)]
    Error,
}

pub fn lex(input: &str) -> Result<Vec<Token>, LaminaError> {
    let lexer = Token::lexer(input);
    lexer.collect::<Result<Vec<_>, _>>()
        .map_err(|_| LaminaError::LexerError("Invalid token".into()))
}
