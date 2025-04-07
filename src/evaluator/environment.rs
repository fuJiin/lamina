use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::LaminaError;
use crate::value::{Environment, NumberKind, Value};

use super::procedures::setup_initial_procedures;

// Set up the initial global environment with basic procedures and special forms
pub fn setup_initial_env() -> Rc<RefCell<Environment>> {
    let env = Rc::new(RefCell::new(Environment {
        parent: None,
        bindings: HashMap::new(),
    }));

    // Add basic procedures
    setup_initial_procedures(&mut env.borrow_mut().bindings);

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
    
    // Note: FFI functions are loaded separately to avoid circular dependencies
    
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

    // Add bytevector operations
    env.borrow_mut().bindings.insert(
        "bytevector".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            let bytes: Result<Vec<u8>, String> = args
                .iter()
                .map(|arg| {
                    if let Value::Number(n) = arg {
                        let value = n.as_f64() as u8;
                        Ok(value)
                    } else {
                        Err("bytevector arguments must be numbers".into())
                    }
                })
                .collect();

            match bytes {
                Ok(bytes) => Ok(Value::Bytevector(Rc::new(RefCell::new(bytes)))),
                Err(e) => Err(e),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "bytevector-length".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("bytevector-length requires exactly one argument".into());
            }

            match &args[0] {
                Value::Bytevector(bv) => {
                    let length = bv.borrow().len();
                    Ok(Value::Number(NumberKind::Integer(length as i64)))
                }
                _ => Err("bytevector-length requires a bytevector".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "bytevector-u8-ref".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 2 {
                return Err("bytevector-u8-ref requires exactly two arguments".into());
            }

            match (&args[0], &args[1]) {
                (Value::Bytevector(bv), Value::Number(n)) => {
                    let index = n.as_f64() as usize;
                    let borrow = bv.borrow();

                    if index >= borrow.len() {
                        return Err(format!(
                            "Index {} out of bounds for bytevector of length {}",
                            index,
                            borrow.len()
                        ));
                    }

                    Ok(Value::Number(NumberKind::Integer(borrow[index] as i64)))
                }
                _ => Err("bytevector-u8-ref requires a bytevector and an index".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "bytevector-u8-set!".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 3 {
                return Err("bytevector-u8-set! requires exactly three arguments".into());
            }

            match (&args[0], &args[1], &args[2]) {
                (Value::Bytevector(bv), Value::Number(n1), Value::Number(n2)) => {
                    let index = n1.as_f64() as usize;
                    let value = n2.as_f64() as u8;
                    let mut borrow = bv.borrow_mut();

                    if index >= borrow.len() {
                        return Err(format!(
                            "Index {} out of bounds for bytevector of length {}",
                            index,
                            borrow.len()
                        ));
                    }

                    borrow[index] = value;
                    Ok(Value::Nil)
                }
                _ => Err("bytevector-u8-set! requires a bytevector, an index, and a value".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "string->utf8".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("string->utf8 requires exactly one argument".into());
            }

            match &args[0] {
                Value::String(s) => {
                    let bytes = s.as_bytes().to_vec();
                    Ok(Value::Bytevector(Rc::new(RefCell::new(bytes))))
                }
                _ => Err("string->utf8 requires a string".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "utf8->string".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("utf8->string requires exactly one argument".into());
            }

            match &args[0] {
                Value::Bytevector(bv) => {
                    let bytes = bv.borrow();
                    match String::from_utf8(bytes.clone()) {
                        Ok(s) => Ok(Value::String(s)),
                        Err(_) => Err("Invalid UTF-8 sequence in bytevector".into()),
                    }
                }
                _ => Err("utf8->string requires a bytevector".into()),
            }
        })),
    );

    // Add string operations
    env.borrow_mut().bindings.insert(
        "string-map".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 2 {
                return Err("string-map requires exactly two arguments".into());
            }

            match (&args[0], &args[1]) {
                (Value::Procedure(proc), Value::String(s)) => {
                    let mut result = String::new();

                    for c in s.chars() {
                        let char_value = Value::Character(c);
                        let mapped = proc(vec![char_value.clone()])?;

                        match mapped {
                            Value::Character(mapped_char) => {
                                result.push(mapped_char);
                            }
                            _ => return Err("string-map procedure must return a character".into()),
                        }
                    }

                    Ok(Value::String(result))
                }
                _ => Err("string-map requires a procedure and a string".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "string-for-each".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 2 {
                return Err("string-for-each requires exactly two arguments".into());
            }

            match (&args[0], &args[1]) {
                (Value::Procedure(proc), Value::String(s)) => {
                    for c in s.chars() {
                        let char_value = Value::Character(c);
                        proc(vec![char_value.clone()])?;
                    }

                    Ok(Value::Nil)
                }
                _ => Err("string-for-each requires a procedure and a string".into()),
            }
        })),
    );

    // Add vector operations
    env.borrow_mut().bindings.insert(
        "vector".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| Ok(Value::Vector(Rc::new(args))))),
    );

    env.borrow_mut().bindings.insert(
        "vector-map".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 2 {
                return Err("vector-map requires exactly two arguments".into());
            }

            match (&args[0], &args[1]) {
                (Value::Procedure(proc), Value::Vector(v)) => {
                    let mut result = Vec::new();

                    for element in v.iter() {
                        let mapped = proc(vec![element.clone()])?;
                        result.push(mapped);
                    }

                    Ok(Value::Vector(Rc::new(result)))
                }
                _ => Err("vector-map requires a procedure and a vector".into()),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "vector-for-each".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 2 {
                return Err("vector-for-each requires exactly two arguments".into());
            }

            match (&args[0], &args[1]) {
                (Value::Procedure(proc), Value::Vector(v)) => {
                    for element in v.iter() {
                        proc(vec![element.clone()])?;
                    }

                    Ok(Value::Nil)
                }
                _ => Err("vector-for-each requires a procedure and a vector".into()),
            }
        })),
    );

    // Add numeric predicates
    env.borrow_mut().bindings.insert(
        "exact-integer?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("exact-integer? requires exactly one argument".into());
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
                return Err("exact? requires exactly one argument".into());
            }

            match &args[0] {
                Value::Number(NumberKind::Integer(_)) => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            }
        })),
    );

    env.borrow_mut().bindings.insert(
        "inexact?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("inexact? requires exactly one argument".into());
            }

            match &args[0] {
                Value::Number(NumberKind::Real(_)) => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            }
        })),
    );

    // Add char-upcase
    env.borrow_mut().bindings.insert(
        "char-upcase".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("char-upcase requires exactly one argument".into());
            }

            match &args[0] {
                Value::Character(c) => {
                    let uppercase = c.to_uppercase().next().unwrap_or(*c);
                    Ok(Value::Character(uppercase))
                }
                _ => Err("char-upcase requires a character".into()),
            }
        })),
    );

    env
}

// Helper function to create an extended environment with new bindings
#[allow(dead_code)]
pub fn extend_environment(
    parent: Rc<RefCell<Environment>>,
    names: Vec<String>,
    values: Vec<Value>,
) -> Result<Rc<RefCell<Environment>>, LaminaError> {
    if names.len() != values.len() {
        return Err(LaminaError::Runtime(format!(
            "Expected {} arguments, got {}",
            names.len(),
            values.len()
        )));
    }

    let new_env = Rc::new(RefCell::new(Environment {
        parent: Some(parent),
        bindings: HashMap::new(),
    }));

    // Add the bindings
    for (name, value) in names.into_iter().zip(values.into_iter()) {
        new_env.borrow_mut().bindings.insert(name, value);
    }

    Ok(new_env)
}

// Function to look up a variable in an environment chain
pub fn lookup_variable(name: &str, env: Rc<RefCell<Environment>>) -> Option<Value> {
    let mut current = env;

    loop {
        // Check if the variable exists in the current environment
        let env_ref = current.borrow();
        if let Some(value) = env_ref.bindings.get(name) {
            return Some(value.clone());
        }

        // If not found, check parent environment
        if let Some(parent) = &env_ref.parent {
            // Move up to parent, dropping the current borrow
            let next = parent.clone();
            drop(env_ref); // Explicitly drop the borrow before reassigning
            current = next;
        } else {
            // No more parents, variable not found
            return None;
        }
    }
}

// Define a variable in the current environment
#[allow(dead_code)]
pub fn define_variable(name: &str, value: Value, env: Rc<RefCell<Environment>>) {
    env.borrow_mut().bindings.insert(name.to_string(), value);
}

// Set the value of an existing variable in the environment chain
#[allow(dead_code)]
pub fn set_variable(
    name: &str,
    value: Value,
    env: Rc<RefCell<Environment>>,
) -> Result<(), LaminaError> {
    let mut current = env;

    loop {
        // Check if the variable exists in the current environment
        let env_ref = current.borrow();
        if env_ref.bindings.contains_key(name) {
            // Variable found, update it
            drop(env_ref); // Explicitly drop the borrow before mutating
            current
                .borrow_mut()
                .bindings
                .insert(name.to_string(), value);
            return Ok(());
        }

        // If not found, check parent environment
        if let Some(parent) = &env_ref.parent {
            // Move up to parent, dropping the current borrow
            let next = parent.clone();
            drop(env_ref); // Explicitly drop the borrow before reassigning
            current = next;
        } else {
            // No more parents, variable not found
            return Err(LaminaError::Runtime(format!(
                "Undefined variable: {}",
                name
            )));
        }
    }
}
