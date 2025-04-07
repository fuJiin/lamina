use std::cell::RefCell;
use std::rc::Rc;

use crate::error::LaminaError;
use crate::value::{Environment, Value};

pub mod environment;
pub mod libraries;
pub mod library_manager;
pub mod procedures;
pub mod special_forms;

// Re-export important functions
pub use environment::setup_initial_env;

// Main evaluation function
pub fn eval(expr: Value) -> Result<Value, LaminaError> {
    let env = setup_initial_env();
    eval_with_env(expr, env)
}

// Evaluation with a specific environment
pub fn eval_with_env(expr: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    match expr {
        Value::Symbol(s) => lookup_symbol(&s, env),
        Value::Number(_)
        | Value::Boolean(_)
        | Value::Character(_)
        | Value::String(_)
        | Value::Nil
        | Value::Procedure(_)
        | Value::RustFn(_, _) => {
            // Self-evaluating expressions
            Ok(expr)
        }
        Value::Pair(_) => eval_pair(expr, env),
        Value::Library(_) => {
            // Libraries are self-evaluating
            Ok(expr)
        }
        Value::RecordType(_) => {
            // Record types are self-evaluating
            Ok(expr)
        }
        Value::Record(_) => {
            // Records are self-evaluating
            Ok(expr)
        }
        Value::Bytevector(_) => {
            // Bytevectors are self-evaluating
            Ok(expr)
        }
        Value::Vector(_) => {
            // Vectors are self-evaluating
            Ok(expr)
        }
        Value::Environment(_) => {
            // Environments are not meant to be evaluated directly
            Err(LaminaError::Runtime(
                "Cannot evaluate an environment".into(),
            ))
        }
    }
}

// Helper function to evaluate pairs
fn eval_pair(expr: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = &expr {
        // Check if the first element is a symbol
        if let Value::Symbol(s) = &pair.0 {
            // Handle special forms
            match s.as_str() {
                "quote" => {
                    if let Value::Pair(quote_pair) = &pair.1 {
                        Ok(quote_pair.0.clone())
                    } else {
                        Err(LaminaError::Runtime("Malformed quote".into()))
                    }
                }
                "lambda" => special_forms::eval_lambda(pair.1.clone(), env),
                "if" => special_forms::eval_if(pair.1.clone(), env),
                "define" => special_forms::eval_define(pair.1.clone(), env),
                "set!" => special_forms::eval_set(pair.1.clone(), env),
                "cond" => special_forms::eval_cond(pair.1.clone(), env),
                "let" => special_forms::eval_let(pair.1.clone(), env),
                "let*" => special_forms::eval_let_star(pair.1.clone(), env),
                "letrec" => special_forms::eval_letrec(pair.1.clone(), env),
                "define-library" => libraries::eval_define_library(pair.1.clone(), env),
                "import" => libraries::eval_import(pair.1.clone(), env),
                "begin" => {
                    let mut result = Value::Nil;
                    let mut current = pair.1.clone();

                    while let Value::Pair(begin_pair) = current {
                        result = eval_with_env(begin_pair.0.clone(), env.clone())?;
                        current = begin_pair.1.clone();
                    }

                    Ok(result)
                }
                "with-exception-handler" => {
                    special_forms::eval_with_exception_handler(pair.1.clone(), env)
                }
                "raise" => special_forms::eval_raise(pair.1.clone(), env),
                "error" => special_forms::eval_error(pair.1.clone(), env),
                "guard" => special_forms::eval_guard(pair.1.clone(), env),
                "define-record-type" => special_forms::eval_define_record_type(pair.1.clone(), env),
                _ => {
                    // Not a special form, evaluate as a procedure call
                    eval_procedure_call(expr, env)
                }
            }
        } else {
            // First element is not a symbol, evaluate as a procedure call
            eval_procedure_call(expr, env)
        }
    } else {
        Err(LaminaError::Runtime("Expected pair".into()))
    }
}

// Helper function to evaluate procedure calls
fn eval_procedure_call(expr: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = expr {
        // Evaluate the operator
        let proc = eval_with_env(pair.0.clone(), env.clone())?;

        // Evaluate the arguments
        let mut args = Vec::new();
        let mut current = pair.1.clone();

        while let Value::Pair(arg_pair) = current {
            let arg = eval_with_env(arg_pair.0.clone(), env.clone())?;
            args.push(arg);
            current = arg_pair.1.clone();
        }

        // Apply the procedure
        match proc {
            Value::Procedure(p) => match p(args) {
                Ok(result) => Ok(result),
                Err(e) => Err(LaminaError::Runtime(e)),
            },
            Value::RustFn(f, name) => match f(args) {
                Ok(result) => Ok(result),
                Err(e) => Err(LaminaError::Runtime(format!("Error in Rust function {}: {}", name, e))),
            },
            _ => Err(LaminaError::Runtime("Not a procedure".into())),
        }
    } else {
        Err(LaminaError::Runtime("Expected pair".into()))
    }
}

// Helper function to look up a symbol in the environment
fn lookup_symbol(name: &str, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    match environment::lookup_variable(name, env) {
        Some(value) => Ok(value),
        None => Err(LaminaError::Runtime(format!(
            "Undefined variable: {}",
            name
        ))),
    }
}
