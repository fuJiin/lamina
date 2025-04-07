use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::value::{Environment, NumberKind, Value};

use super::libraries;
use super::special_forms::register_special_forms;

// Function to create a new environment with optional parent
pub fn create_environment(parent: Option<Rc<RefCell<Environment>>>) -> Rc<RefCell<Environment>> {
    let mut env = Environment::new();
    env.parent = parent;
    Rc::new(RefCell::new(env))
}

/// Setup the initial environment with standard procedures and special forms
pub fn setup_initial_env() -> Rc<RefCell<Environment>> {
    let env = create_environment(None);

    // Register special forms
    register_special_forms(env.clone());

    // Register standard procedures
    register_procedures(env.clone());

    // Add a marker for environment type
    env.borrow_mut().bindings.insert(
        "environment-type".to_string(),
        Value::Symbol("standard".to_string()),
    );

    // Add boolean constants
    env.borrow_mut()
        .bindings
        .insert("#t".to_string(), Value::Boolean(true));
    env.borrow_mut()
        .bindings
        .insert("#f".to_string(), Value::Boolean(false));
    env.borrow_mut()
        .bindings
        .insert("else".to_string(), Value::Boolean(true));

    // Register libraries (EVM, etc.)
    if let Err(e) = libraries::setup_libraries(env.clone()) {
        eprintln!("Warning: Failed to setup libraries: {}", e);
    }

    env
}

// Register basic procedures (+ - * / etc.)
#[allow(dead_code)]
pub fn register_procedures(env: Rc<RefCell<Environment>>) {
    // Define standard arithmetic operators
    env.borrow_mut().bindings.insert(
        "+".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            let mut sum = 0.0;
            for arg in args {
                match arg {
                    Value::Number(n) => sum += n.as_f64(),
                    _ => return Err("+ requires numeric arguments".into()),
                }
            }
            Ok(Value::from(sum))
        })),
    );

    // Define subtraction
    env.borrow_mut().bindings.insert(
        "-".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.is_empty() {
                return Err("- requires at least one argument".into());
            }

            if args.len() == 1 {
                // Negation
                match &args[0] {
                    Value::Number(n) => match n {
                        NumberKind::Integer(i) => Ok(Value::Number(NumberKind::Integer(-i))),
                        NumberKind::Real(r) => Ok(Value::Number(NumberKind::Real(-r))),
                        NumberKind::Rational(num, den) => {
                            Ok(Value::Number(NumberKind::Rational(-num, *den)))
                        }
                    },
                    _ => Err("- requires numeric arguments".into()),
                }
            } else {
                // Subtraction
                let mut result = match &args[0] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err("- requires numeric arguments".into()),
                };

                for arg in args.iter().skip(1) {
                    match arg {
                        Value::Number(n) => result -= n.as_f64(),
                        _ => return Err("- requires numeric arguments".into()),
                    }
                }

                Ok(Value::from(result))
            }
        })),
    );

    // Define multiplication
    env.borrow_mut().bindings.insert(
        "*".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            let mut product = 1.0;
            for arg in args {
                match arg {
                    Value::Number(n) => product *= n.as_f64(),
                    _ => return Err("* requires numeric arguments".into()),
                }
            }
            Ok(Value::from(product))
        })),
    );

    // Define division
    env.borrow_mut().bindings.insert(
        "/".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.is_empty() {
                return Err("/ requires at least one argument".into());
            }

            if args.len() == 1 {
                // Reciprocal
                match &args[0] {
                    Value::Number(n) => {
                        let value = n.as_f64();
                        if value == 0.0 {
                            return Err("Division by zero".into());
                        }
                        Ok(Value::from(1.0 / value))
                    }
                    _ => Err("/ requires numeric arguments".into()),
                }
            } else {
                // Division
                let mut result = match &args[0] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err("/ requires numeric arguments".into()),
                };

                for arg in args.iter().skip(1) {
                    match arg {
                        Value::Number(n) => {
                            let value = n.as_f64();
                            if value == 0.0 {
                                return Err("Division by zero".into());
                            }
                            result /= value;
                        }
                        _ => return Err("/ requires numeric arguments".into()),
                    }
                }

                Ok(Value::from(result))
            }
        })),
    );

    // Define equal for numbers
    env.borrow_mut().bindings.insert(
        "=".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("= requires at least two arguments".into());
            }

            let first = match &args[0] {
                Value::Number(n) => n.as_f64(),
                _ => return Err("= requires numeric arguments".into()),
            };

            for arg in args.iter().skip(1) {
                match arg {
                    Value::Number(n) => {
                        if (n.as_f64() - first).abs() > f64::EPSILON {
                            return Ok(Value::Boolean(false));
                        }
                    }
                    _ => return Err("= requires numeric arguments".into()),
                }
            }

            Ok(Value::Boolean(true))
        })),
    );

    // Less than
    env.borrow_mut().bindings.insert(
        "<".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("< requires at least two arguments".into());
            }

            for i in 0..args.len() - 1 {
                let a = match &args[i] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err("< requires numeric arguments".into()),
                };

                let b = match &args[i + 1] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err("< requires numeric arguments".into()),
                };

                if a >= b {
                    return Ok(Value::Boolean(false));
                }
            }

            Ok(Value::Boolean(true))
        })),
    );

    // Greater than
    env.borrow_mut().bindings.insert(
        ">".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("> requires at least two arguments".into());
            }

            for i in 0..args.len() - 1 {
                let a = match &args[i] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err("> requires numeric arguments".into()),
                };

                let b = match &args[i + 1] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err("> requires numeric arguments".into()),
                };

                if a <= b {
                    return Ok(Value::Boolean(false));
                }
            }

            Ok(Value::Boolean(true))
        })),
    );

    // Less than or equal
    env.borrow_mut().bindings.insert(
        "<=".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("<= requires at least two arguments".into());
            }

            for i in 0..args.len() - 1 {
                let a = match &args[i] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err("<= requires numeric arguments".into()),
                };

                let b = match &args[i + 1] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err("<= requires numeric arguments".into()),
                };

                if a > b {
                    return Ok(Value::Boolean(false));
                }
            }

            Ok(Value::Boolean(true))
        })),
    );

    // Greater than or equal
    env.borrow_mut().bindings.insert(
        ">=".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err(">= requires at least two arguments".into());
            }

            for i in 0..args.len() - 1 {
                let a = match &args[i] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err(">= requires numeric arguments".into()),
                };

                let b = match &args[i + 1] {
                    Value::Number(n) => n.as_f64(),
                    _ => return Err(">= requires numeric arguments".into()),
                };

                if a < b {
                    return Ok(Value::Boolean(false));
                }
            }

            Ok(Value::Boolean(true))
        })),
    );

    // Define boolean operations
    env.borrow_mut().bindings.insert(
        "not".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("not requires exactly one argument".into());
            }
            match args[0] {
                Value::Boolean(b) => Ok(Value::Boolean(!b)),
                _ => Ok(Value::Boolean(false)), // All non-#f values are truthy in Scheme
            }
        })),
    );

    // Add 'and' special form
    env.borrow_mut().bindings.insert(
        "and".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.is_empty() {
                return Ok(Value::Boolean(true)); // (and) => #t
            }

            let mut result = Value::Boolean(true);
            for arg in args {
                if let Value::Boolean(false) = arg {
                    return Ok(Value::Boolean(false)); // Short-circuit if any arg is #f
                }
                result = arg; // Return last value
            }
            Ok(result)
        })),
    );

    // Add 'or' special form
    env.borrow_mut().bindings.insert(
        "or".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.is_empty() {
                return Ok(Value::Boolean(false)); // (or) => #f
            }

            for arg in args {
                if let Value::Boolean(false) = arg {
                    continue; // Skip #f values
                }
                return Ok(arg); // Return first truthy value
            }
            Ok(Value::Boolean(false)) // No truthy values found
        })),
    );

    // Add basic list operations
    env.borrow_mut().bindings.insert(
        "cons".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 2 {
                return Err("cons requires exactly 2 arguments".into());
            }
            Ok(Value::cons(args[0].clone(), args[1].clone()))
        })),
    );

    env.borrow_mut().bindings.insert(
        "car".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("car requires exactly 1 argument".into());
            }
            match &args[0] {
                Value::Pair(pair) => Ok(pair.0.clone()),
                _ => Err("car requires a pair".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "cdr".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("cdr requires exactly 1 argument".into());
            }
            match &args[0] {
                Value::Pair(pair) => Ok(pair.1.clone()),
                _ => Err("cdr requires a pair".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "list".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            let mut result = Value::Nil;
            for arg in args.iter().rev() {
                result = Value::cons(arg.clone(), result);
            }
            Ok(result)
        })),
    );

    env.borrow_mut().bindings.insert(
        "null?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("null? requires exactly 1 argument".into());
            }
            match &args[0] {
                Value::Nil => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "pair?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("pair? requires exactly 1 argument".into());
            }
            match &args[0] {
                Value::Pair(_) => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            }
        })),
    );

    // Add bytevector operations
    env.borrow_mut().bindings.insert(
        "bytevector".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            let mut bytes = Vec::new();
            for arg in &args {
                if let Value::Number(n) = arg {
                    match n.to_u8() {
                        Ok(byte) => bytes.push(byte),
                        Err(e) => return Err(e),
                    }
                } else {
                    return Err("bytevector requires numeric arguments".into());
                }
            }
            Ok(Value::Bytevector(Rc::new(RefCell::new(bytes))))
        })),
    );

    env.borrow_mut().bindings.insert(
        "bytevector-length".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("bytevector-length requires exactly 1 argument".into());
            }

            match &args[0] {
                Value::Bytevector(bytes) => {
                    let len = bytes.borrow().len();
                    Ok(Value::Number(NumberKind::Integer(len as i64)))
                }
                _ => Err("bytevector-length requires a bytevector".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "bytevector-u8-ref".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 2 {
                return Err("bytevector-u8-ref requires exactly 2 arguments".into());
            }

            let bv = match &args[0] {
                Value::Bytevector(bytes) => bytes.clone(),
                _ => return Err("bytevector-u8-ref requires a bytevector as first argument".into()),
            };

            let index = match &args[1] {
                Value::Number(NumberKind::Integer(i)) => *i as usize,
                _ => return Err("bytevector-u8-ref requires an integer as second argument".into()),
            };

            let bytes = bv.borrow();
            if index >= bytes.len() {
                return Err(format!("bytevector-u8-ref: index out of bounds: {}", index));
            }

            Ok(Value::Number(NumberKind::Integer(bytes[index] as i64)))
        })),
    );

    env.borrow_mut().bindings.insert(
        "bytevector-u8-set!".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 3 {
                return Err("bytevector-u8-set! requires exactly 3 arguments".into());
            }

            let bv = match &args[0] {
                Value::Bytevector(bytes) => bytes.clone(),
                _ => {
                    return Err("bytevector-u8-set! requires a bytevector as first argument".into())
                }
            };

            let index = match &args[1] {
                Value::Number(NumberKind::Integer(i)) => *i as usize,
                _ => return Err("bytevector-u8-set! requires an integer as second argument".into()),
            };

            let value = match &args[2] {
                Value::Number(n) => match n {
                    NumberKind::Integer(i) => *i as u8,
                    NumberKind::Real(r) => *r as u8,
                    NumberKind::Rational(num, den) => (*num as f64 / *den as f64) as u8,
                },
                _ => {
                    return Err(
                        "bytevector-u8-set! requires a numeric value as third argument".into(),
                    )
                }
            };

            let mut bytes = bv.borrow_mut();
            if index >= bytes.len() {
                return Err(format!(
                    "bytevector-u8-set!: index out of bounds: {}",
                    index
                ));
            }

            bytes[index] = value;
            Ok(Value::Nil)
        })),
    );

    // Add string operations
    env.borrow_mut().bindings.insert(
        "string-map".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("string-map requires at least 2 arguments".into());
            }

            let proc = &args[0];
            let string = match &args[1] {
                Value::String(s) => s,
                _ => return Err("string-map requires a string as second argument".into()),
            };

            let chars: Vec<char> = string.chars().collect();
            let mut result = String::new();

            for c in chars {
                let char_val = Value::Character(c);
                let result_val = match proc {
                    Value::Procedure(p) => p(vec![char_val.clone()])?,
                    _ => return Err("string-map requires a procedure as first argument".into()),
                };

                match result_val {
                    Value::Character(c) => result.push(c),
                    _ => return Err("procedure must return a character".into()),
                }
            }

            Ok(Value::String(result))
        })),
    );

    // Add character operations
    env.borrow_mut().bindings.insert(
        "char-upcase".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("char-upcase requires exactly 1 argument".into());
            }

            match &args[0] {
                Value::Character(c) => {
                    let upper = c.to_uppercase().next().unwrap_or(*c);
                    Ok(Value::Character(upper))
                }
                _ => Err("char-upcase requires a character".into()),
            }
        })),
    );

    // String operations
    env.borrow_mut().bindings.insert(
        "string->utf8".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("string->utf8 requires exactly one argument".into());
            }
            if let Value::String(s) = &args[0] {
                let bytes = s.as_bytes().to_vec();
                Ok(Value::Bytevector(Rc::new(RefCell::new(bytes))))
            } else {
                Err("string->utf8 requires a string argument".into())
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "utf8->string".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("utf8->string requires exactly one argument".into());
            }
            if let Value::Bytevector(bv) = &args[0] {
                let bytes = bv.borrow();
                match std::str::from_utf8(&bytes) {
                    Ok(s) => Ok(Value::String(s.to_string())),
                    Err(_) => Err("invalid UTF-8 sequence".into()),
                }
            } else {
                Err("utf8->string requires a bytevector argument".into())
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "string-for-each".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("string-for-each requires at least two arguments".into());
            }

            let proc = &args[0];
            if !matches!(proc, Value::Procedure(_) | Value::RustFn(_, _)) {
                return Err("First argument to string-for-each must be a procedure".into());
            }

            // Check that all remaining arguments are strings
            let mut strings = Vec::new();
            for arg in &args[1..] {
                if let Value::String(s) = arg {
                    strings.push(s.clone());
                } else {
                    return Err("All arguments after the procedure must be strings".into());
                }
            }

            // Check that all strings have the same length
            let str_len = strings[0].chars().count();
            for s in &strings[1..] {
                if s.chars().count() != str_len {
                    return Err("All strings must have the same length".into());
                }
            }

            // Convert strings to vectors of characters
            let char_vecs: Vec<Vec<char>> = strings.iter().map(|s| s.chars().collect()).collect();

            // Apply procedure to each set of characters
            for i in 0..str_len {
                let mut char_args = Vec::new();
                for chars in &char_vecs {
                    char_args.push(Value::Character(chars[i]));
                }

                // Call the procedure with the characters
                match proc {
                    Value::Procedure(p) => {
                        match p(char_args) {
                            Ok(_) => {} // Ignore the result
                            Err(e) => return Err(e),
                        }
                    }
                    Value::RustFn(f, _) => {
                        match f(char_args) {
                            Ok(_) => {} // Ignore the result
                            Err(e) => return Err(e),
                        }
                    }
                    _ => unreachable!(), // We checked this above
                }
            }

            Ok(Value::Nil)
        })),
    );

    // Vector operations
    env.borrow_mut().bindings.insert(
        "vector".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| Ok(Value::Vector(Rc::new(args))))),
    );

    env.borrow_mut().bindings.insert(
        "vector-length".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("vector-length requires exactly 1 argument".into());
            }

            match &args[0] {
                Value::Vector(v) => {
                    let len = v.len();
                    Ok(Value::Number(NumberKind::Integer(len as i64)))
                }
                _ => Err("vector-length requires a vector".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "vector-ref".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 2 {
                return Err("vector-ref requires exactly 2 arguments".into());
            }

            let v = match &args[0] {
                Value::Vector(vec) => vec.clone(),
                _ => return Err("vector-ref requires a vector as first argument".into()),
            };

            let index = match &args[1] {
                Value::Number(NumberKind::Integer(i)) => *i as usize,
                _ => return Err("vector-ref requires an integer as second argument".into()),
            };

            if index >= v.len() {
                return Err(format!("vector-ref: index out of bounds: {}", index));
            }

            Ok(v[index].clone())
        })),
    );

    env.borrow_mut().bindings.insert(
        "vector-map".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("vector-map requires at least 2 arguments".into());
            }

            let proc = &args[0];
            if !matches!(proc, Value::Procedure(_) | Value::RustFn(_, _)) {
                return Err("First argument to vector-map must be a procedure".into());
            }

            // Check that all remaining arguments are vectors
            let mut vectors = Vec::new();
            for arg in &args[1..] {
                if let Value::Vector(v) = arg {
                    vectors.push(v.clone());
                } else {
                    return Err("All arguments after the procedure must be vectors".into());
                }
            }

            // Check that all vectors have the same length
            let vector_len = vectors[0].len();
            for v in &vectors[1..] {
                if v.len() != vector_len {
                    return Err("All vectors must have the same length".into());
                }
            }

            // Apply procedure to each set of elements
            let mut result_vector = Vec::new();
            for i in 0..vector_len {
                let mut element_args = Vec::new();
                for v in &vectors {
                    element_args.push(v[i].clone());
                }

                // Call the procedure with the elements
                let result_val = match proc {
                    Value::Procedure(p) => p(element_args)?,
                    Value::RustFn(f, _) => f(element_args)?,
                    _ => unreachable!(), // We checked this above
                };

                result_vector.push(result_val);
            }

            Ok(Value::Vector(Rc::new(result_vector)))
        })),
    );

    env.borrow_mut().bindings.insert(
        "vector-for-each".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("vector-for-each requires at least 2 arguments".into());
            }

            let proc = &args[0];
            if !matches!(proc, Value::Procedure(_) | Value::RustFn(_, _)) {
                return Err("First argument to vector-for-each must be a procedure".into());
            }

            // Check that all remaining arguments are vectors
            let mut vectors = Vec::new();
            for arg in &args[1..] {
                if let Value::Vector(v) = arg {
                    vectors.push(v.clone());
                } else {
                    return Err("All arguments after the procedure must be vectors".into());
                }
            }

            // Check that all vectors have the same length
            let vector_len = vectors[0].len();
            for v in &vectors[1..] {
                if v.len() != vector_len {
                    return Err("All vectors must have the same length".into());
                }
            }

            // Apply procedure to each set of elements
            for i in 0..vector_len {
                let mut element_args = Vec::new();
                for v in &vectors {
                    element_args.push(v[i].clone());
                }

                // Call the procedure with the elements
                match proc {
                    Value::Procedure(p) => {
                        match p(element_args) {
                            Ok(_) => {} // Ignore the result
                            Err(e) => return Err(e),
                        }
                    }
                    Value::RustFn(f, _) => {
                        match f(element_args) {
                            Ok(_) => {} // Ignore the result
                            Err(e) => return Err(e),
                        }
                    }
                    _ => unreachable!(), // We checked this above
                }
            }

            Ok(Value::Nil)
        })),
    );

    // Add numeric predicates
    env.borrow_mut().bindings.insert(
        "exact-integer?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("exact-integer? requires exactly 1 argument".into());
            }

            match &args[0] {
                Value::Number(NumberKind::Integer(_)) => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "exact?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("exact? requires exactly 1 argument".into());
            }

            match &args[0] {
                Value::Number(NumberKind::Integer(_)) => Ok(Value::Boolean(true)),
                Value::Number(NumberKind::Rational(_, _)) => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "inexact?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("inexact? requires exactly 1 argument".into());
            }

            match &args[0] {
                Value::Number(NumberKind::Real(_)) => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            }
        })),
    );
}

// Create a child environment by extending the parent with new bindings
#[allow(dead_code)]
pub fn extend_environment(
    parent: Rc<RefCell<Environment>>,
    names: Vec<String>,
    values: Vec<Value>,
) -> Result<Rc<RefCell<Environment>>, Error> {
    if names.len() != values.len() {
        return Err(Error::Runtime(format!(
            "Expected {} arguments, got {}",
            names.len(),
            values.len()
        )));
    }

    let mut env = Environment::new();
    env.parent = Some(parent);

    for (name, value) in names.into_iter().zip(values.into_iter()) {
        env.bindings.insert(name, value);
    }

    Ok(Rc::new(RefCell::new(env)))
}

// Look up a variable in the environment chain
pub fn lookup_variable(name: &str, env: Rc<RefCell<Environment>>) -> Result<Value, String> {
    let mut current_env = env;

    loop {
        // Check the current environment
        let env_ref = current_env.borrow();
        if let Some(value) = env_ref.bindings.get(name) {
            return Ok(value.clone());
        }

        // Move to parent environment if there is one
        match &env_ref.parent {
            Some(parent) => {
                let parent_clone = parent.clone();
                drop(env_ref); // Drop the borrow before reassigning
                current_env = parent_clone;
            }
            None => {
                drop(env_ref); // Drop the borrow
                return Err(format!("Undefined variable: {}", name));
            }
        }
    }
}

// Set a variable's value in the environment chain
#[allow(dead_code)]
pub fn set_variable(name: &str, value: Value, env: Rc<RefCell<Environment>>) -> Result<(), Error> {
    let mut current_env = env;

    loop {
        // Check the current environment
        let env_ref = current_env.borrow();
        let found = env_ref.bindings.contains_key(name);

        if found {
            drop(env_ref); // Drop the borrow before mutating
            current_env
                .borrow_mut()
                .bindings
                .insert(name.to_string(), value);
            return Ok(());
        }

        // Move to parent environment if there is one
        match &env_ref.parent {
            Some(parent) => {
                let parent_clone = parent.clone();
                drop(env_ref); // Drop the borrow before reassigning
                current_env = parent_clone;
            }
            None => {
                drop(env_ref); // Drop the borrow
                return Err(Error::Runtime(format!("Undefined variable: {}", name)));
            }
        }
    }
}

// Define a new variable in the current environment
#[allow(dead_code)]
pub fn define_variable(name: &str, value: Value, env: &mut Environment) {
    env.bindings.insert(name.to_string(), value);
}
