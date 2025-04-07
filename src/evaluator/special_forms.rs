use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::LaminaError;
use crate::value::{Environment, Record, RecordType, Value};

use super::eval_with_env;

// Lambda special form
pub fn eval_lambda(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        let params = pair.0.clone();

        // Get the body expression (it's the car of pair.1)
        let body = if let Value::Pair(body_pair) = &pair.1 {
            body_pair.0.clone()
        } else {
            // This should not happen with well-formed expressions
            return Err(LaminaError::Runtime("Malformed lambda".into()));
        };

        let env_clone = env.clone();
        Ok(Value::Procedure(Rc::new(move |args: Vec<Value>| {
            let new_env = Rc::new(RefCell::new(Environment {
                parent: Some(env_clone.clone()),
                bindings: HashMap::new(),
            }));

            // Bind parameters
            let mut param_list = params.clone();
            let mut arg_idx = 0;
            while let Value::Pair(param_pair) = param_list {
                if let Value::Symbol(name) = &param_pair.0 {
                    if arg_idx >= args.len() {
                        return Err(format!(
                            "Too few arguments, expected {} got {}",
                            arg_idx + 1,
                            args.len()
                        ));
                    }
                    new_env
                        .borrow_mut()
                        .bindings
                        .insert(name.clone(), args[arg_idx].clone());
                }
                param_list = param_pair.1.clone();
                arg_idx += 1;
            }
            if let Value::Nil = param_list {
                // This is fine, we've bound all parameters
            } else if let Value::Symbol(name) = param_list {
                // This is a rest parameter
                new_env.borrow_mut().bindings.insert(name, Value::Nil);
            } else {
                return Err("Invalid parameter list".into());
            }

            // Evaluate body
            match eval_with_env(body.clone(), new_env) {
                Ok(result) => Ok(result),
                Err(e) => Err(e.to_string()),
            }
        })))
    } else {
        Err(LaminaError::Runtime("Invalid lambda form".into()))
    }
}

// If special form
pub fn eval_if(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(test_pair) = args {
        let test = eval_with_env(test_pair.0.clone(), env.clone())?;
        if let Value::Pair(conseq_pair) = &test_pair.1 {
            match test {
                Value::Boolean(false) => {
                    if let Value::Pair(alt_pair) = &conseq_pair.1 {
                        eval_with_env(alt_pair.0.clone(), env)
                    } else {
                        Ok(Value::Nil)
                    }
                }
                _ => eval_with_env(conseq_pair.0.clone(), env),
            }
        } else {
            Err(LaminaError::Runtime("Malformed if expression".into()))
        }
    } else {
        Err(LaminaError::Runtime("Malformed if expression".into()))
    }
}

// Define special form
pub fn eval_define(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        match &pair.0 {
            Value::Symbol(name) => {
                // Get the actual value expression (it's the car of pair.1)
                let value_expr = if let Value::Pair(val_pair) = &pair.1 {
                    val_pair.0.clone()
                } else {
                    // This should not happen with well-formed expressions
                    return Err(LaminaError::Runtime("Malformed define".into()));
                };

                // Evaluate the value expression
                let value = eval_with_env(value_expr, env.clone())?;

                env.borrow_mut().bindings.insert(name.clone(), value);
                Ok(Value::Nil)
            }
            Value::Pair(proc_pair) => {
                // For function definitions like (define (func x) body)
                if let Value::Symbol(name) = &proc_pair.0 {
                    let params = proc_pair.1.clone();
                    let body = pair.1.clone();
                    let env_clone = env.clone();
                    let proc = Value::Procedure(Rc::new(move |args: Vec<Value>| {
                        let new_env = Rc::new(RefCell::new(Environment {
                            parent: Some(env_clone.clone()),
                            bindings: HashMap::new(),
                        }));

                        // Bind parameters
                        let mut param_list = params.clone();
                        let mut arg_idx = 0;
                        while let Value::Pair(param_pair) = param_list {
                            if let Value::Symbol(param_name) = &param_pair.0 {
                                if arg_idx >= args.len() {
                                    return Err(format!(
                                        "Too few arguments, expected {} got {}",
                                        arg_idx + 1,
                                        args.len()
                                    ));
                                }
                                new_env
                                    .borrow_mut()
                                    .bindings
                                    .insert(param_name.clone(), args[arg_idx].clone());
                            }
                            param_list = param_pair.1.clone();
                            arg_idx += 1;
                        }

                        // Evaluate body
                        match eval_with_env(body.clone(), new_env) {
                            Ok(result) => Ok(result),
                            Err(e) => Err(e.to_string()),
                        }
                    }));
                    env.borrow_mut().bindings.insert(name.clone(), proc);
                    Ok(Value::Nil)
                } else {
                    Err(LaminaError::Runtime(
                        "First argument to define must be a symbol".into(),
                    ))
                }
            }
            _ => Err(LaminaError::Runtime(
                "First argument to define must be a symbol".into(),
            )),
        }
    } else {
        Err(LaminaError::Runtime("Malformed define".into()))
    }
}

// Set! special form
pub fn eval_set(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        if let Value::Symbol(name) = &pair.0 {
            // Get the actual value expression (it's the car of pair.1)
            let value_expr = if let Value::Pair(val_pair) = &pair.1 {
                val_pair.0.clone()
            } else {
                // This should not happen with well-formed expressions
                return Err(LaminaError::Runtime("Malformed set!".into()));
            };

            // Evaluate the value expression
            let value = eval_with_env(value_expr, env.clone())?;

            let mut current = env;
            let mut target_env = None;

            // First, find the environment that contains the variable
            while target_env.is_none() {
                let env_ref = current.borrow();
                if env_ref.bindings.contains_key(name) {
                    target_env = Some(current.clone());
                } else if let Some(parent) = &env_ref.parent {
                    let next = parent.clone();
                    drop(env_ref); // Explicitly drop the borrow before reassigning
                    current = next;
                } else {
                    return Err(LaminaError::Runtime(format!(
                        "Undefined variable: {}",
                        name
                    )));
                }
            }

            // Then, update the variable in the found environment
            if let Some(env) = target_env {
                env.borrow_mut().bindings.insert(name.clone(), value);
                Ok(Value::Nil)
            } else {
                Err(LaminaError::Runtime(format!(
                    "Undefined variable: {}",
                    name
                )))
            }
        } else {
            Err(LaminaError::Runtime(
                "First argument to set! must be a symbol".into(),
            ))
        }
    } else {
        Err(LaminaError::Runtime("Malformed set!".into()))
    }
}

// Cond special form
pub fn eval_cond(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    let mut current = args;
    while let Value::Pair(pair) = current {
        let clause = &pair.0;
        if let Value::Pair(clause_pair) = clause {
            let test = eval_with_env(clause_pair.0.clone(), env.clone())?;
            match test {
                Value::Boolean(false) => {
                    current = pair.1.clone();
                    continue;
                }
                _ => {
                    if let Value::Pair(conseq_pair) = &clause_pair.1 {
                        return eval_with_env(conseq_pair.0.clone(), env);
                    } else {
                        return Ok(test);
                    }
                }
            }
        } else if let Value::Symbol(s) = clause {
            if s == "else" {
                if let Value::Pair(else_pair) = &pair.1 {
                    return eval_with_env(else_pair.0.clone(), env);
                } else {
                    return Ok(Value::Nil);
                }
            }
        }
        current = pair.1.clone();
    }
    Ok(Value::Nil)
}

// Let special form
pub fn eval_let(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        let bindings = pair.0.clone();

        // Get the body expression (it's the car of pair.1)
        let body = if let Value::Pair(body_pair) = &pair.1 {
            body_pair.0.clone()
        } else {
            // This should not happen with well-formed expressions
            return Err(LaminaError::Runtime("Malformed let".into()));
        };

        // Create new environment
        let new_env = Rc::new(RefCell::new(Environment {
            parent: Some(env.clone()),
            bindings: HashMap::new(),
        }));

        // Evaluate bindings
        let mut current = bindings;
        while let Value::Pair(binding_pair) = current {
            if let Value::Pair(var_pair) = &binding_pair.0 {
                if let Value::Symbol(name) = &var_pair.0 {
                    // Get the value expression (it's the car of var_pair.1)
                    let value_expr = if let Value::Pair(val_pair) = &var_pair.1 {
                        val_pair.0.clone()
                    } else {
                        // This should not happen with well-formed expressions
                        return Err(LaminaError::Runtime("Malformed binding in let".into()));
                    };

                    let value = eval_with_env(value_expr, env.clone())?;
                    new_env.borrow_mut().bindings.insert(name.clone(), value);
                }
            }
            current = binding_pair.1.clone();
        }

        // Evaluate body
        eval_with_env(body, new_env)
    } else {
        Err(LaminaError::Runtime("Malformed let".into()))
    }
}

// Let* special form
pub fn eval_let_star(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        let bindings = pair.0.clone();

        // Get the body expression (it's the car of pair.1)
        let body = if let Value::Pair(body_pair) = &pair.1 {
            body_pair.0.clone()
        } else {
            // This should not happen with well-formed expressions
            return Err(LaminaError::Runtime("Malformed let*".into()));
        };

        // Create new environment
        let mut current_env = env.clone();

        // Evaluate bindings sequentially
        let mut current = bindings;
        while let Value::Pair(binding_pair) = current {
            if let Value::Pair(var_pair) = &binding_pair.0 {
                if let Value::Symbol(name) = &var_pair.0 {
                    // Get the value expression (it's the car of var_pair.1)
                    let value_expr = if let Value::Pair(val_pair) = &var_pair.1 {
                        val_pair.0.clone()
                    } else {
                        // This should not happen with well-formed expressions
                        return Err(LaminaError::Runtime("Malformed binding in let*".into()));
                    };

                    let value = eval_with_env(value_expr, current_env.clone())?;

                    let new_env = Rc::new(RefCell::new(Environment {
                        parent: Some(current_env.clone()),
                        bindings: HashMap::new(),
                    }));
                    new_env.borrow_mut().bindings.insert(name.clone(), value);
                    current_env = new_env;
                }
            }
            current = binding_pair.1.clone();
        }

        // Evaluate body
        eval_with_env(body, current_env)
    } else {
        Err(LaminaError::Runtime("Malformed let*".into()))
    }
}

// Letrec special form
pub fn eval_letrec(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        let bindings = pair.0.clone();

        // Get the body expression (it's the car of pair.1)
        let body = if let Value::Pair(body_pair) = &pair.1 {
            body_pair.0.clone()
        } else {
            // This should not happen with well-formed expressions
            return Err(LaminaError::Runtime("Malformed letrec".into()));
        };

        // Create new environment
        let new_env = Rc::new(RefCell::new(Environment {
            parent: Some(env.clone()),
            bindings: HashMap::new(),
        }));

        // First pass: create bindings with undefined values
        let mut current = bindings.clone();
        while let Value::Pair(binding_pair) = current {
            if let Value::Pair(var_pair) = &binding_pair.0 {
                if let Value::Symbol(name) = &var_pair.0 {
                    new_env
                        .borrow_mut()
                        .bindings
                        .insert(name.clone(), Value::Nil);
                }
            }
            current = binding_pair.1.clone();
        }

        // Second pass: evaluate bindings in the new environment
        let mut current = bindings;
        while let Value::Pair(binding_pair) = current {
            if let Value::Pair(var_pair) = &binding_pair.0 {
                if let Value::Symbol(name) = &var_pair.0 {
                    // Get the value expression (it's the car of var_pair.1)
                    let value_expr = if let Value::Pair(val_pair) = &var_pair.1 {
                        val_pair.0.clone()
                    } else {
                        // This should not happen with well-formed expressions
                        return Err(LaminaError::Runtime("Malformed binding in letrec".into()));
                    };

                    let value = eval_with_env(value_expr, new_env.clone())?;
                    new_env.borrow_mut().bindings.insert(name.clone(), value);
                }
            }
            current = binding_pair.1.clone();
        }

        // Evaluate body
        eval_with_env(body, new_env)
    } else {
        Err(LaminaError::Runtime("Malformed letrec".into()))
    }
}

// Exception handling implementation
pub fn eval_with_exception_handler(
    args: Value,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, LaminaError> {
    if let Value::Pair(handler_pair) = args {
        let handler = eval_with_env(handler_pair.0.clone(), env.clone())?;

        if let Value::Pair(ref thunk_pair) = handler_pair.1 {
            let thunk = eval_with_env(thunk_pair.0.clone(), env.clone())?;

            match thunk {
                Value::Procedure(f) => {
                    // Try to call the thunk procedure with no arguments
                    match f(vec![]) {
                        Ok(result) => Ok(result),
                        Err(e) => {
                            // If the thunk raises an exception, call the handler with the exception object
                            if let Value::Procedure(h) = handler {
                                // Create a simple exception value from the error message
                                let exception = Value::Symbol(e);
                                match h(vec![exception]) {
                                    Ok(result) => Ok(result),
                                    Err(new_e) => Err(LaminaError::Runtime(new_e)),
                                }
                            } else {
                                Err(LaminaError::Runtime("Handler must be a procedure".into()))
                            }
                        }
                    }
                }
                _ => Err(LaminaError::Runtime("Thunk must be a procedure".into())),
            }
        } else {
            Err(LaminaError::Runtime(
                "with-exception-handler requires a handler and a thunk".into(),
            ))
        }
    } else {
        Err(LaminaError::Runtime(
            "with-exception-handler requires a handler and a thunk".into(),
        ))
    }
}

pub fn eval_raise(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        // Evaluate the argument
        let exception = eval_with_env(pair.0.clone(), env)?;

        // Raise the exception
        Err(LaminaError::Runtime(format!("Exception: {:?}", exception)))
    } else {
        Err(LaminaError::Runtime("raise requires an argument".into()))
    }
}

pub fn eval_error(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        // Evaluate the arguments
        let message = eval_with_env(pair.0.clone(), env.clone())?;

        // Format the error message
        let error_msg = match message {
            Value::String(s) => s,
            _ => format!("{:?}", message),
        };

        // Raise the error
        Err(LaminaError::Runtime(format!("Error: {}", error_msg)))
    } else {
        Err(LaminaError::Runtime("error requires an argument".into()))
    }
}

pub fn eval_guard(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(var_clauses_pair) = args {
        // Extract the variable and clauses
        if let Value::Pair(var_pair) = &var_clauses_pair.0 {
            let exception_var = match &var_pair.0 {
                Value::Symbol(s) => s.clone(),
                _ => {
                    return Err(LaminaError::Runtime(
                        "Guard variable must be a symbol".into(),
                    ));
                }
            };

            // Get the clauses
            let clauses = var_pair.1.clone();

            // Get the body
            if let Value::Pair(body_pair) = &var_clauses_pair.1 {
                let body = body_pair.0.clone();

                // Try to evaluate the body
                match eval_with_env(body, env.clone()) {
                    Ok(result) => Ok(result),
                    Err(error) => {
                        // An exception occurred, create a new environment with the exception bound to the variable
                        let guard_env = Rc::new(RefCell::new(Environment {
                            parent: Some(env.clone()),
                            bindings: HashMap::new(),
                        }));

                        // Create an exception value from the error
                        let exception_value = match error {
                            LaminaError::Runtime(e) => {
                                if e.starts_with("Exception: ") {
                                    // This is from a 'raise' call, extract the actual value
                                    let symbol_content = e.trim_start_matches("Exception: ");
                                    Value::Symbol(symbol_content.to_string())
                                } else {
                                    Value::Symbol(e)
                                }
                            }
                            _ => Value::Symbol(format!("{:?}", error)),
                        };

                        // Bind the exception to the variable
                        guard_env
                            .borrow_mut()
                            .bindings
                            .insert(exception_var, exception_value.clone());

                        // Evaluate the clauses
                        let mut current = clauses;
                        while let Value::Pair(clause_pair) = current {
                            let clause = &clause_pair.0;

                            if let Value::Pair(test_pair) = clause {
                                // Evaluate the test
                                let test = eval_with_env(test_pair.0.clone(), guard_env.clone())?;

                                match test {
                                    Value::Boolean(true) => {
                                        // Test passed, evaluate the expression
                                        if let Value::Pair(expr_pair) = &test_pair.1 {
                                            return eval_with_env(expr_pair.0.clone(), guard_env);
                                        }
                                    }
                                    Value::Boolean(false) => {
                                        // Test failed, try next clause
                                        current = clause_pair.1.clone();
                                        continue;
                                    }
                                    _ => {
                                        return Err(LaminaError::Runtime(
                                            "Guard test must evaluate to a boolean".into(),
                                        ));
                                    }
                                }
                            } else if let Value::Symbol(s) = clause {
                                if s == "else" {
                                    // Else clause, always matches
                                    if let Value::Pair(expr_pair) = &clause_pair.1 {
                                        if let Value::Pair(expr) = &expr_pair.0 {
                                            return eval_with_env(expr.0.clone(), guard_env);
                                        } else {
                                            return eval_with_env(expr_pair.0.clone(), guard_env);
                                        }
                                    }
                                }
                            }

                            current = clause_pair.1.clone();
                        }

                        // No matching clause, re-raise the exception
                        Err(LaminaError::Runtime(format!(
                            "Unhandled exception: {:?}",
                            exception_value
                        )))
                    }
                }
            } else {
                Err(LaminaError::Runtime("Malformed guard expression".into()))
            }
        } else {
            Err(LaminaError::Runtime("Malformed guard expression".into()))
        }
    } else {
        Err(LaminaError::Runtime("Malformed guard expression".into()))
    }
}

// Implement define-record-type form
pub fn eval_define_record_type(
    args: Value,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, LaminaError> {
    if let Value::Pair(type_pair) = args {
        // Get the record type name
        let type_name = match &type_pair.0 {
            Value::Symbol(name) => name.clone(),
            _ => {
                return Err(LaminaError::Runtime(
                    "Record type name must be a symbol".into(),
                ));
            }
        };

        // Get the constructor expression
        if let Value::Pair(ctor_pair) = &type_pair.1 {
            let constructor = match &ctor_pair.0 {
                Value::Symbol(ctor) => ctor.clone(),
                Value::Pair(ctor_spec) => {
                    if let Value::Symbol(ctor_name) = &ctor_spec.0 {
                        ctor_name.clone()
                    } else {
                        return Err(LaminaError::Runtime(
                            "Constructor name must be a symbol".into(),
                        ));
                    }
                }
                _ => {
                    return Err(LaminaError::Runtime(
                        "Invalid constructor specification".into(),
                    ));
                }
            };

            // Get constructor parameters
            let mut constructor_fields = Vec::new();
            if let Value::Pair(ctor_spec) = &ctor_pair.0 {
                let mut current = ctor_spec.1.clone();
                while let Value::Pair(param_pair) = current {
                    if let Value::Symbol(param) = &param_pair.0 {
                        constructor_fields.push(param.clone());
                    } else {
                        return Err(LaminaError::Runtime(
                            "Constructor parameter must be a symbol".into(),
                        ));
                    }
                    current = param_pair.1.clone();
                }
            }

            // Get the predicate name
            if let Value::Pair(pred_pair) = &ctor_pair.1 {
                let predicate = match &pred_pair.0 {
                    Value::Symbol(pred) => pred.clone(),
                    _ => {
                        return Err(LaminaError::Runtime("Predicate must be a symbol".into()));
                    }
                };

                // Process field specifications
                let mut fields = Vec::new();
                let mut current = pred_pair.1.clone();

                while let Value::Pair(field_pair) = current {
                    if let Value::Pair(field_spec) = &field_pair.0 {
                        // Get field name
                        let field_name = match &field_spec.0 {
                            Value::Symbol(name) => name.clone(),
                            _ => {
                                return Err(LaminaError::Runtime(
                                    "Field name must be a symbol".into(),
                                ));
                            }
                        };

                        // Get accessor name
                        if let Value::Pair(accessor_pair) = &field_spec.1 {
                            let accessor = match &accessor_pair.0 {
                                Value::Symbol(acc) => acc.clone(),
                                _ => {
                                    return Err(LaminaError::Runtime(
                                        "Accessor must be a symbol".into(),
                                    ));
                                }
                            };

                            // Check if there's a mutator
                            let mutator = if let Value::Pair(mutator_pair) = &accessor_pair.1 {
                                match &mutator_pair.0 {
                                    Value::Symbol(mut_name) => Some(mut_name.clone()),
                                    _ => {
                                        return Err(LaminaError::Runtime(
                                            "Mutator must be a symbol".into(),
                                        ));
                                    }
                                }
                            } else {
                                None
                            };

                            fields.push((field_name, accessor, mutator));
                        } else {
                            return Err(LaminaError::Runtime(
                                "Field specification must include an accessor".into(),
                            ));
                        }
                    } else {
                        return Err(LaminaError::Runtime("Invalid field specification".into()));
                    }

                    current = field_pair.1.clone();
                }

                // Create the record type
                let record_type = Rc::new(RecordType {
                    name: type_name.clone(),
                    fields: fields
                        .iter()
                        .map(|(name, _, mutator)| (name.clone(), mutator.is_some()))
                        .collect(),
                });

                // Define the constructor
                let record_type_clone = record_type.clone();
                let constructor_fields_clone = constructor_fields.clone();
                let constructor_clone = constructor.clone();
                let constructor_proc = Value::Procedure(Rc::new(move |args: Vec<Value>| {
                    if args.len() != constructor_fields_clone.len() {
                        return Err(format!(
                            "Constructor {} requires {} arguments, got {}",
                            constructor_clone,
                            constructor_fields_clone.len(),
                            args.len()
                        ));
                    }

                    let record = Rc::new(Record {
                        type_info: record_type_clone.clone(),
                        values: RefCell::new(HashMap::new()),
                    });

                    // Set the initial values from constructor arguments
                    for (i, field) in constructor_fields_clone.iter().enumerate() {
                        for (name, _) in &record_type_clone.fields {
                            if name == field {
                                record
                                    .values
                                    .borrow_mut()
                                    .insert(field.clone(), args[i].clone());
                                break;
                            }
                        }
                    }

                    Ok(Value::Record(record))
                }));

                // Define the predicate
                let type_name_clone = type_name.clone();
                let predicate_clone = predicate.clone();
                let predicate_proc = Value::Procedure(Rc::new(move |args: Vec<Value>| {
                    if args.len() != 1 {
                        return Err(format!(
                            "Predicate {} requires exactly 1 argument",
                            predicate_clone
                        ));
                    }

                    match &args[0] {
                        Value::Record(record) => {
                            Ok(Value::Boolean(record.type_info.name == type_name_clone))
                        }
                        _ => Ok(Value::Boolean(false)),
                    }
                }));

                // Define accessors and mutators for each field
                let mut accessors = Vec::new();
                let mut mutators = Vec::new();

                for (field_name, accessor_name, mutator_name) in fields {
                    let field_name_clone = field_name.clone();
                    let type_name_clone = type_name.clone();
                    let accessor_name_clone = accessor_name.clone();

                    // Create accessor
                    let accessor_proc = Value::Procedure(Rc::new(move |args: Vec<Value>| {
                        if args.len() != 1 {
                            return Err(format!(
                                "Accessor {} requires exactly 1 argument",
                                accessor_name_clone
                            ));
                        }

                        match &args[0] {
                            Value::Record(record) => {
                                if record.type_info.name != type_name_clone {
                                    return Err(format!(
                                        "Expected record of type {}, got {}",
                                        type_name_clone, record.type_info.name
                                    ));
                                }

                                if let Some(value) = record.values.borrow().get(&field_name_clone) {
                                    Ok(value.clone())
                                } else {
                                    Err(format!("Field {} not found in record", field_name_clone))
                                }
                            }
                            _ => Err(format!("Expected record, got {:?}", args[0])),
                        }
                    }));

                    accessors.push((accessor_name, accessor_proc));

                    // Create mutator if specified
                    if let Some(mutator) = mutator_name {
                        let field_name_clone = field_name.clone();
                        let type_name_clone = type_name.clone();
                        let mutator_clone = mutator.clone();

                        let mutator_proc = Value::Procedure(Rc::new(move |args: Vec<Value>| {
                            if args.len() != 2 {
                                return Err(format!(
                                    "Mutator {} requires exactly 2 arguments",
                                    mutator_clone
                                ));
                            }

                            match &args[0] {
                                Value::Record(record) => {
                                    if record.type_info.name != type_name_clone {
                                        return Err(format!(
                                            "Expected record of type {}, got {}",
                                            type_name_clone, record.type_info.name
                                        ));
                                    }

                                    // Check if the field is mutable
                                    let is_mutable =
                                        record.type_info.fields.iter().any(|(name, mutable)| {
                                            name == &field_name_clone && *mutable
                                        });

                                    if !is_mutable {
                                        return Err(format!(
                                            "Field {} is not mutable",
                                            field_name_clone
                                        ));
                                    }

                                    record
                                        .values
                                        .borrow_mut()
                                        .insert(field_name_clone.clone(), args[1].clone());
                                    Ok(Value::Nil)
                                }
                                _ => Err(format!("Expected record, got {:?}", args[0])),
                            }
                        }));

                        mutators.push((mutator, mutator_proc));
                    }
                }

                // Add the type, constructor, predicate, accessors, and mutators to the environment
                env.borrow_mut()
                    .bindings
                    .insert(type_name, Value::RecordType(record_type));
                env.borrow_mut()
                    .bindings
                    .insert(constructor, constructor_proc);
                env.borrow_mut().bindings.insert(predicate, predicate_proc);

                for (name, proc) in accessors {
                    env.borrow_mut().bindings.insert(name, proc);
                }

                for (name, proc) in mutators {
                    env.borrow_mut().bindings.insert(name, proc);
                }

                Ok(Value::Nil)
            } else {
                Err(LaminaError::Runtime(
                    "Malformed record type definition".into(),
                ))
            }
        } else {
            Err(LaminaError::Runtime(
                "Malformed record type definition".into(),
            ))
        }
    } else {
        Err(LaminaError::Runtime(
            "Malformed record type definition".into(),
        ))
    }
}
