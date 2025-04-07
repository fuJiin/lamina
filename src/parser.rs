use crate::error::Error;
use crate::lexer::Token;
use crate::value::{Value, NumberKind};
use std::rc::Rc;

// Helper function to parse a number string into a NumberKind
fn parse_number(n: String) -> Result<NumberKind, Error> {
    if n.contains('.') {
        match n.parse::<f64>() {
            Ok(f) => Ok(NumberKind::Real(f)),
            Err(_) => Err(Error::Parser(format!("Invalid number: {}", n))),
        }
    } else {
        match n.parse::<i64>() {
            Ok(i) => Ok(NumberKind::Integer(i)),
            Err(_) => Err(Error::Parser(format!("Invalid number: {}", n))),
        }
    }
}

pub fn parse(tokens: &[Token]) -> Result<Value, Error> {
    if tokens.is_empty() {
        return Err(Error::Parser("No tokens to parse".to_string()));
    }

    let (expr, pos) = parse_expr(tokens, 0)?;
    if pos != tokens.len() {
        return Err(Error::Parser("Extra tokens at end of input".to_string()));
    }

    Ok(expr)
}

fn parse_expr(tokens: &[Token], pos: usize) -> Result<(Value, usize), Error> {
    if pos >= tokens.len() {
        return Err(Error::Parser("Unexpected end of input".to_string()));
    }

    match &tokens[pos] {
        Token::LeftParen => parse_list(tokens, pos + 1),
        Token::RightParen => Err(Error::Parser("Unexpected right parenthesis".to_string())),
        Token::Quote => {
            let (quoted_expr, new_pos) = parse_expr(tokens, pos + 1)?;
            let quote_sym = Value::Symbol("quote".to_string());
            let quoted_pair = Rc::new((quoted_expr, Value::Nil));
            let result = Value::Pair(Rc::new((quote_sym, Value::Pair(quoted_pair))));
            Ok((result, new_pos))
        }
        Token::Symbol(s) => Ok((Value::Symbol(s.clone()), pos + 1)),
        Token::Number(n) => {
            let num_kind = parse_number(n.clone())?;
            Ok((Value::Number(num_kind), pos + 1))
        },
        Token::String(s) => Ok((Value::String(s.clone()), pos + 1)),
        Token::TrueValue => Ok((Value::Boolean(true), pos + 1)),
        Token::FalseValue => Ok((Value::Boolean(false), pos + 1)),
        Token::Character(c) => {
            let ch = match c.as_str() {
                "space" => ' ',
                "newline" => '\n',
                s if s.len() == 1 => s.chars().next().unwrap(),
                _ => return Err(Error::Parser(format!("Invalid character: {}", c))),
            };
            Ok((Value::Character(ch), pos + 1))
        },
        Token::Error => Err(Error::Parser("Invalid token".to_string())),
    }
}

fn parse_list(tokens: &[Token], pos: usize) -> Result<(Value, usize), Error> {
    if pos >= tokens.len() {
        return Err(Error::Parser("Unexpected end of input in list".to_string()));
    }

    match &tokens[pos] {
        Token::RightParen => Ok((Value::Nil, pos + 1)),
        _ => {
            let (car, new_pos) = parse_expr(tokens, pos)?;
            if new_pos >= tokens.len() {
                return Err(Error::Parser("Unexpected end of input in list".to_string()));
            }
            let (cdr, final_pos) = parse_list(tokens, new_pos)?;
            Ok((Value::Pair(Rc::new((car, cdr))), final_pos))
        }
    }
}
