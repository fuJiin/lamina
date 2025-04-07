use crate::error::LaminaError;
use crate::lexer::Token;
use crate::value::{NumberKind, Value};
use std::rc::Rc;

pub struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            self.position += 1;
            Some(&self.tokens[self.position - 1])
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
                    return Ok(elements
                        .into_iter()
                        .rev()
                        .fold(Value::Nil, |acc, val| Value::Pair(Rc::new((val, acc)))));
                }
                _ => elements.push(self.parse_expr()?),
            }
        }

        Err(LaminaError::Parser("Unclosed parenthesis".into()))
    }

    fn parse_expr(&mut self) -> Result<Value, LaminaError> {
        match self.peek() {
            Some(Token::LParen) => self.parse_list(),
            Some(Token::Quote) => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Value::Pair(Rc::new((
                    Value::Symbol("quote".into()),
                    Value::Pair(Rc::new((expr, Value::Nil))),
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
                let num_str = n.clone();
                self.advance();
                if let Ok(i) = num_str.parse::<i64>() {
                    Ok(Value::Number(NumberKind::Integer(i)))
                } else if let Ok(f) = num_str.parse::<f64>() {
                    Ok(Value::Number(NumberKind::Real(f)))
                } else {
                    Err(LaminaError::Parser("Invalid number".into()))
                }
            }
            Some(Token::String(s)) => {
                let s_clone = s.clone();
                self.advance();
                Ok(Value::String(s_clone))
            }
            Some(Token::Symbol(s)) => {
                let s_clone = s.clone();
                self.advance();
                Ok(Value::Symbol(s_clone))
            }
            Some(Token::Character(c)) => {
                let c_clone = *c;
                self.advance();
                Ok(Value::Character(c_clone))
            }
            Some(Token::Space) => {
                self.advance();
                Ok(Value::Character(' '))
            }
            None => Err(LaminaError::Parser("Unexpected end of input".into())),
            _ => Err(LaminaError::Parser("Unexpected token".into())),
        }
    }
}

pub fn parse(tokens: &[Token]) -> Result<Value, LaminaError> {
    let mut parser = Parser::new(tokens);
    let result = parser.parse_expr()?;

    // Make sure we've consumed all tokens
    if parser.peek().is_some() {
        return Err(LaminaError::Parser("Extra tokens after parsing".into()));
    }

    Ok(result)
}
