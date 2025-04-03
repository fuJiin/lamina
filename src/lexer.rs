use logos::Logos;
#[derive(Logos, Debug, Clone)]
pub enum Token {
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("#t")]
    True,

    #[token("#f")]
    False,

    #[regex("[0-9]+|[0-9]+\\.[0-9]+", |lex| lex.slice().to_string())]
    Number(String),

    #[regex("[a-zA-Z+\\-*/<>=!?_][a-zA-Z0-9+\\-*/<>=!?_]*", |lex| lex.slice().to_string())]
    Symbol(String),

    #[regex("#\\\\space")]
    Space,

    #[regex("#\\\\[a-zA-Z]+", |lex| lex.slice()[2..].chars().next(), priority = 2)]
    #[regex("#\\\\.", |lex| lex.slice().chars().nth(2), priority = 1)]
    Character(char),

    #[regex("\"([^\"]|\\\\.)*\"", |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    String(String),

    #[token("'")]
    Quote,
    
    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[regex(r";[^\n]*\n", logos::skip)]
    Error,
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let lexer = Token::lexer(input);
    lexer.collect::<Result<_, _>>()
        .map_err(|_| "Invalid token".to_string())
}