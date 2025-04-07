use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::value::{Environment, Library, NumberKind, Value};

use super::environment::create_environment;

// Helper functions for EVM library
pub fn check_args_count(func_name: &str, args: &[Value], expected: usize) -> Result<(), String> {
    if args.len() != expected {
        return Err(format!(
            "{} expected {} arguments, got {}",
            func_name,
            expected,
            args.len()
        ));
    }
    Ok(())
}

pub fn number_to_i64(val: &Value) -> Result<i64, String> {
    match val {
        Value::Number(n) => match n {
            NumberKind::Integer(i) => Ok(*i),
            NumberKind::Real(r) => Ok(*r as i64),
            NumberKind::Rational(num, den) => Ok(*num / *den),
        },
        _ => Err(format!("Expected number, got {}", val)),
    }
}

// Base library registration
pub fn register_base_library(env: Rc<RefCell<Environment>>) {
    let base_env = create_environment(Some(env.clone()));

    // Add basic list operations
    base_env.borrow_mut().bindings.insert(
        "append".to_string(),
        Value::Procedure(Rc::new(|_args| {
            // Implementation of append
            Ok(Value::Nil)
        })),
    );

    // Register the library in the parent environment
    env.borrow_mut().bindings.insert(
        "base".to_string(),
        Value::Library(Rc::new(RefCell::new(Library {
            name: vec!["scheme".to_string(), "base".to_string()],
            exports: vec!["append".to_string()],
            imports: vec![],
            environment: base_env,
        }))),
    );
}

// File library registration
pub fn register_file_library(env: Rc<RefCell<Environment>>) {
    let file_env = create_environment(Some(env.clone()));

    // Add file operations
    file_env.borrow_mut().bindings.insert(
        "file-exists?".to_string(),
        Value::Procedure(Rc::new(|_args| {
            // Implementation of file-exists?
            Ok(Value::Boolean(false))
        })),
    );

    // Register the library in the parent environment
    env.borrow_mut().bindings.insert(
        "file".to_string(),
        Value::Library(Rc::new(RefCell::new(Library {
            name: vec!["scheme".to_string(), "file".to_string()],
            exports: vec!["file-exists?".to_string()],
            imports: vec![],
            environment: file_env,
        }))),
    );
}

// Math library registration
pub fn register_math_library(env: Rc<RefCell<Environment>>) {
    let math_env = create_environment(Some(env.clone()));

    // Add math operations
    math_env.borrow_mut().bindings.insert(
        "abs".to_string(),
        Value::Procedure(Rc::new(|args| {
            check_args_count("abs", &args, 1)?;
            match &args[0] {
                Value::Number(n) => match n {
                    NumberKind::Integer(i) => Ok(Value::Number(NumberKind::Integer(i.abs()))),
                    NumberKind::Real(r) => Ok(Value::Number(NumberKind::Real(r.abs()))),
                    NumberKind::Rational(num, den) => {
                        Ok(Value::Number(NumberKind::Rational(num.abs(), *den)))
                    }
                },
                _ => Err("abs: expected number".to_string()),
            }
        })),
    );

    // Register the library in the parent environment
    env.borrow_mut().bindings.insert(
        "math".to_string(),
        Value::Library(Rc::new(RefCell::new(Library {
            name: vec!["scheme".to_string(), "math".to_string()],
            exports: vec!["abs".to_string()],
            imports: vec![],
            environment: math_env,
        }))),
    );
}

// EVM library registration
pub fn register_evm_library(env: Rc<RefCell<Environment>>) {
    let evm_env = create_environment(Some(env.clone()));

    // Storage operations
    evm_env.borrow_mut().bindings.insert(
        "storage-load".to_string(),
        Value::Procedure(Rc::new(|args| {
            check_args_count("storage-load", &args, 1)?;
            let _slot = number_to_i64(&args[0])?;
            // This is a mock implementation since we're focusing on compilation
            Ok(Value::Number(NumberKind::Integer(0)))
        })),
    );

    evm_env.borrow_mut().bindings.insert(
        "storage-store".to_string(),
        Value::Procedure(Rc::new(|args| {
            check_args_count("storage-store", &args, 2)?;
            let _slot = number_to_i64(&args[0])?;
            let _value = &args[1];
            // This is a mock implementation since we're focusing on compilation
            Ok(Value::Nil)
        })),
    );

    // Contract execution control
    evm_env.borrow_mut().bindings.insert(
        "revert".to_string(),
        Value::Procedure(Rc::new(|args| {
            check_args_count("revert", &args, 1)?;
            // This is a mock implementation since we're focusing on compilation
            Ok(Value::Nil)
        })),
    );

    // Register the library in the parent environment
    env.borrow_mut().bindings.insert(
        "evm".to_string(),
        Value::Library(Rc::new(RefCell::new(Library {
            name: vec!["evm".to_string()],
            exports: vec![
                "storage-load".to_string(),
                "storage-store".to_string(),
                "revert".to_string(),
            ],
            imports: vec![],
            environment: evm_env,
        }))),
    );
}

// Setup all libraries
pub fn setup_libraries(env: Rc<RefCell<Environment>>) -> Result<(), Error> {
    register_base_library(env.clone());
    register_file_library(env.clone());
    register_math_library(env.clone());
    register_evm_library(env.clone());
    Ok(())
}
