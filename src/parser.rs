
use crate::error::LaminaError;
use crate::lexer::Token;
use crate::value::{Value, NumberKind};
use std::rc::Rc;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            self.position += 1;
            Some(self.tokens[self.position - 1].clone())
        } else {
            None
        }
    }

    fn parse_list(&mut self) -> Result<Value, LaminaError> {
        self.advance(); // consume opening paren
        let mut elements = Vec::new();
        
        while let Some(token) = self.peek() {
            match token {
                Token::RParen => {
                    self.advance();
                    return Ok(elements.into_iter().rev().fold(
                        Value::Nil,
                        |acc, val| Value::Pair(Rc::new((val, acc)))
                    ));
                }
                _ => elements.push(self.parse_expr()?),
            }
        }
        
        Err(LaminaError::ParserError("Unclosed parenthesis".into()))
    }

    fn parse_expr(&mut self) -> Result<Value, LaminaError> {
        let token = self.peek().cloned();
        match token {
            Some(Token::LParen) => self.parse_list(),
            Some(Token::Quote) => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Value::Pair(Rc::new((
                    Value::Symbol("quote".into()),
                    Value::Pair(Rc::new((expr, Value::Nil)))
                ))))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Value::Boolean(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Value::Boolean(false))
            }
            Some(Token::Number(n)) => {
                self.advance();
                if let Ok(i) = n.parse::<i64>() {
                    Ok(Value::Number(NumberKind::Integer(i)))
                } else if let Ok(f) = n.parse::<f64>() {
                    Ok(Value::Number(NumberKind::Real(f)))
                } else {
                    Err(LaminaError::ParserError("Invalid number".into()))
                }
            }
            Some(Token::String(s)) => {
                self.advance();
                Ok(Value::String(s.clone()))
            }
            Some(Token::Symbol(s)) => {
                self.advance();
                Ok(Value::Symbol(s.clone()))
            }
            Some(Token::Character(c)) => {
                self.advance();
                Ok(Value::Character(c))
            }
            None => Err(LaminaError::ParserError("Unexpected end of input".into())),
            _ => Err(LaminaError::ParserError("Unexpected token".into())),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Value, LaminaError> {
    let mut parser = Parser::new(tokens);
    parser.parse_expr()
}
