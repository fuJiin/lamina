
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::error::LaminaError;
use crate::value::{Value, NumberKind, Environment};

pub fn eval(expr: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let env = Rc::new(RefCell::new(Environment {
        parent: None,
        bindings: setup_initial_env(),
    }));
    eval_with_env(expr, env).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

fn eval_with_env(expr: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    match expr {
        Value::Symbol(s) => lookup_symbol(&s, env),
        Value::Pair(pair) => eval_pair(pair.clone(), env),
        Value::Number(_) | Value::Boolean(_) | Value::String(_) | Value::Character(_) | 
        Value::Vector(_) | Value::Procedure(_) => Ok(expr),
        Value::Nil => Ok(Value::Nil),
        Value::Environment(_) => Err(LaminaError::RuntimeError("Cannot evaluate environment".into())),
    }
}

fn eval_pair(pair: (Value, Value), env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    match pair.0 {
        Value::Symbol(s) => match s.as_str() {
            "quote" => Ok(car(pair.1)?),
            "lambda" => eval_lambda(pair.1, env),
            "if" => eval_if(pair.1, env),
            "define" => eval_define(pair.1, env),
            "set!" => eval_set(pair.1, env),
            _ => eval_procedure(pair, env),
        },
        _ => eval_procedure(pair, env),
    }
}

fn setup_initial_env() -> HashMap<String, Value> {
    let mut env = HashMap::new();
    
    // Basic arithmetic
    env.insert("+".to_string(), make_binary_op(|a, b| a + b));
    env.insert("-".to_string(), make_binary_op(|a, b| a - b));
    env.insert("*".to_string(), make_binary_op(|a, b| a * b));
    env.insert("/".to_string(), make_binary_op(|a, b| a / b));
    
    // Comparisons
    env.insert("=".to_string(), make_comparison_op(|a, b| a == b));
    env.insert("<".to_string(), make_comparison_op(|a, b| a < b));
    env.insert(">".to_string(), make_comparison_op(|a, b| a > b));
    
    // List operations
    env.insert("car".to_string(), make_procedure(car));
    env.insert("cdr".to_string(), make_procedure(cdr));
    env.insert("cons".to_string(), make_procedure(cons));
    
    env
}

fn make_binary_op<F>(op: F) -> Value 
where F: Fn(i64, i64) -> i64 + 'static {
    Value::Procedure(Rc::new(move |args: Vec<Value>| {
        if args.len() != 2 {
            return Err("Binary operation requires exactly 2 arguments".into());
        }
        match (&args[0], &args[1]) {
            (Value::Number(NumberKind::Integer(a)), Value::Number(NumberKind::Integer(b))) => {
                Ok(Value::Number(NumberKind::Integer(op(*a, *b))))
            }
            _ => Err("Arguments must be integers".into()),
        }
    }))
}

fn make_comparison_op<F>(op: F) -> Value 
where F: Fn(i64, i64) -> bool + 'static {
    Value::Procedure(Rc::new(move |args: Vec<Value>| {
        if args.len() != 2 {
            return Err("Comparison requires exactly 2 arguments".into());
        }
        match (&args[0], &args[1]) {
            (Value::Number(NumberKind::Integer(a)), Value::Number(NumberKind::Integer(b))) => {
                Ok(Value::Boolean(op(*a, *b)))
            }
            _ => Err("Arguments must be integers".into()),
        }
    }))
}

fn make_procedure<F>(f: F) -> Value 
where F: Fn(Value) -> Result<Value, String> + 'static {
    Value::Procedure(Rc::new(move |args: Vec<Value>| {
        if args.len() != 1 {
            return Err("Procedure requires exactly 1 argument".into());
        }
        f(args[0].clone())
    }))
}

fn car(v: Value) -> Result<Value, String> {
    match v {
        Value::Pair(p) => Ok(p.0.clone()),
        _ => Err("car: expected pair".into()),
    }
}

fn cdr(v: Value) -> Result<Value, String> {
    match v {
        Value::Pair(p) => Ok(p.1.clone()),
        _ => Err("cdr: expected pair".into()),
    }
}

fn cons(arg: Value) -> Result<Value, String> {
    if let Value::Pair(p) = arg {
        Ok(Value::Pair(Rc::new((p.0.clone(), p.1.clone()))))
    } else {
        Err("cons: expected pair".into())
    }
}

fn lookup_symbol(name: &str, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    let env_ref = env.borrow();
    if let Some(value) = env_ref.bindings.get(name) {
        Ok(value.clone())
    } else if let Some(parent) = &env_ref.parent {
        lookup_symbol(name, parent.clone())
    } else {
        Err(LaminaError::RuntimeError(format!("Undefined symbol: {}", name)))
    }
}

fn eval_lambda(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    match args {
        Value::Pair(pair) => {
            let params = pair.0;
            let body = pair.1;
            Ok(Value::Procedure(Rc::new(move |args: Vec<Value>| {
                // Create new environment for lambda
                let new_env = Rc::new(RefCell::new(Environment {
                    parent: Some(env.clone()),
                    bindings: HashMap::new(),
                }));
                
                // Bind parameters
                let mut param_list = params.clone();
                let mut arg_idx = 0;
                while let Value::Pair(pair) = param_list {
                    if let Value::Symbol(name) = &pair.0 {
                        new_env.borrow_mut().bindings.insert(name.clone(), args[arg_idx].clone());
                    }
                    param_list = pair.1;
                    arg_idx += 1;
                }
                
                // Evaluate body
                match eval_with_env(body.clone(), new_env) {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            })))
        }
        _ => Err(LaminaError::RuntimeError("Invalid lambda form".into())),
    }
}

fn eval_if(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(test_pair) = args {
        let test = eval_with_env(test_pair.0, env.clone())?;
        if let Value::Pair(conseq_pair) = test_pair.1 {
            match test {
                Value::Boolean(false) => {
                    if let Value::Pair(alt_pair) = conseq_pair.1 {
                        eval_with_env(alt_pair.0, env)
                    } else {
                        Ok(Value::Nil)
                    }
                }
                _ => eval_with_env(conseq_pair.0, env),
            }
        } else {
            Err(LaminaError::RuntimeError("Malformed if expression".into()))
        }
    } else {
        Err(LaminaError::RuntimeError("Malformed if expression".into()))
    }
}

fn eval_define(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        if let Value::Symbol(name) = pair.0 {
            let value = eval_with_env(pair.1, env.clone())?;
            env.borrow_mut().bindings.insert(name, value);
            Ok(Value::Nil)
        } else {
            Err(LaminaError::RuntimeError("First argument to define must be a symbol".into()))
        }
    } else {
        Err(LaminaError::RuntimeError("Malformed define".into()))
    }
}

fn eval_set(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    if let Value::Pair(pair) = args {
        if let Value::Symbol(name) = pair.0 {
            let value = eval_with_env(pair.1, env.clone())?;
            let mut current = env;
            loop {
                let env_ref = current.borrow();
                if env_ref.bindings.contains_key(&name) {
                    drop(env_ref);
                    current.borrow_mut().bindings.insert(name, value);
                    return Ok(Value::Nil);
                }
                if let Some(parent) = &env_ref.parent {
                    current = parent.clone();
                } else {
                    return Err(LaminaError::RuntimeError(format!("Undefined variable: {}", name)));
                }
            }
        } else {
            Err(LaminaError::RuntimeError("First argument to set! must be a symbol".into()))
        }
    } else {
        Err(LaminaError::RuntimeError("Malformed set!".into()))
    }
}

fn eval_procedure(pair: (Value, Value), env: Rc<RefCell<Environment>>) -> Result<Value, LaminaError> {
    let proc = eval_with_env(pair.0, env.clone())?;
    let mut args = Vec::new();
    let mut rest = pair.1;
    while let Value::Pair(p) = rest {
        args.push(eval_with_env(p.0, env.clone())?);
        rest = p.1;
    }
    
    match proc {
        Value::Procedure(f) => f(args).map_err(|e| LaminaError::RuntimeError(e)),
        _ => Err(LaminaError::RuntimeError("Not a procedure".into())),
    }
}
