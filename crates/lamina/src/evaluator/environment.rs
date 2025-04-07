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
