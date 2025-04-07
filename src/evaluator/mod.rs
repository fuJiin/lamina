use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::value::{Environment, Value};

// Make these public
pub mod environment;
pub mod libraries;
pub mod library_manager;
pub mod procedures;
pub mod special_forms;

/// Evaluate a Lamina expression
pub fn eval(expr: Value) -> Result<Value, Error> {
    // Create a fresh environment
    let env = environment::setup_initial_env();
    eval_with_env(expr, env)
}

/// Evaluate a Lamina expression in a given environment
pub fn eval_with_env(expr: Value, env: Rc<RefCell<Environment>>) -> Result<Value, Error> {
    match expr {
        Value::Symbol(s) => {
            // Look up the symbol in the environment
            environment::lookup_variable(&s, env.clone())
                .map_err(|e| Error::Runtime(e))
        }
        Value::Pair(pair) => {
            // Get the operator (first element of the list)
            let op = &pair.0;
            let args = pair.1.clone();

            // Check if it's a special form
            if let Value::Symbol(s) = op {
                match s.as_str() {
                    "lambda" => special_forms::eval_lambda(args, env),
                    "if" => special_forms::eval_if(args, env),
                    "define" => special_forms::eval_define(args, env),
                    "set!" => special_forms::eval_set(args, env),
                    "cond" => special_forms::eval_cond(args, env),
                    "let" => special_forms::eval_let(args, env),
                    "let*" => special_forms::eval_let_star(args, env),
                    "letrec" => special_forms::eval_letrec(args, env),
                    "with-exception-handler" => {
                        special_forms::eval_with_exception_handler(args, env)
                    }
                    "raise" => special_forms::eval_raise(args, env),
                    "error" => special_forms::eval_error(args, env),
                    "guard" => special_forms::eval_guard(args, env),
                    "define-record-type" => special_forms::eval_define_record_type(args, env),
                    "begin" => eval_begin(args, env),
                    _ => {
                        // It's a function call
                        // Evaluate the operator
                        let op_val = eval_with_env(op.clone(), env.clone())?;

                        // Evaluate the arguments
                        let mut arg_values = Vec::new();
                        let mut remaining_args = args;
                        while let Value::Pair(arg_pair) = remaining_args {
                            let arg_val = eval_with_env(arg_pair.0.clone(), env.clone())?;
                            arg_values.push(arg_val);
                            remaining_args = arg_pair.1.clone();
                        }

                        // Apply the function to the arguments
                        apply(op_val, arg_values)
                    }
                }
            } else {
                // Evaluate the operator
                let op_val = eval_with_env(op.clone(), env.clone())?;

                // Evaluate the arguments
                let mut arg_values = Vec::new();
                let mut remaining_args = args;
                while let Value::Pair(arg_pair) = remaining_args {
                    let arg_val = eval_with_env(arg_pair.0.clone(), env.clone())?;
                    arg_values.push(arg_val);
                    remaining_args = arg_pair.1.clone();
                }

                // Apply the function to the arguments
                apply(op_val, arg_values)
            }
        }
        // Self-evaluating forms
        Value::Number(_) | Value::String(_) | Value::Boolean(_) | Value::Character(_)
        | Value::Vector(_) | Value::Nil | Value::Bytevector(_) => Ok(expr),

        // Other forms
        Value::Procedure(_) => Ok(expr),
        Value::RustFn(_, _) => Ok(expr),
        Value::Library(_) => Ok(expr),
        Value::RecordType(_) => Ok(expr),
        Value::Record(_) => Ok(expr),
        Value::Environment(_) => Ok(expr),
    }
}

/// Apply a function to arguments
fn apply(func: Value, args: Vec<Value>) -> Result<Value, Error> {
    match func {
        Value::Procedure(p) => p(args).map_err(|e| Error::Runtime(e)),
        Value::RustFn(f, _) => f(args).map_err(|e| Error::Runtime(e)),
        _ => Err(Error::Runtime(format!("Not a function: {:?}", func))),
    }
}

// Evaluate a begin expression (sequence of expressions)
fn eval_begin(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, Error> {
    let mut result = Value::Nil;
    let mut remaining_args = args;
    
    while let Value::Pair(pair) = remaining_args {
        result = eval_with_env(pair.0.clone(), env.clone())?;
        remaining_args = pair.1.clone();
    }
    
    Ok(result)
}
