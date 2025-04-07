use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{NumberKind, Value};

// Set up all the standard Scheme procedures
pub fn setup_initial_procedures(env: &mut HashMap<String, Value>) {
    // Arithmetic operations
    env.insert(
        "+".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            let mut sum = 0.0;
            for arg in args {
                if let Value::Number(num) = arg {
                    sum += num.as_f64();
                } else {
                    return Err("+ requires numeric arguments".into());
                }
            }
            Ok(Value::from(sum))
        })),
    );

    env.insert(
        "-".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.is_empty() {
                return Err("- requires at least one argument".into());
            }

            if args.len() == 1 {
                if let Value::Number(num) = &args[0] {
                    return Ok(Value::from(-num.as_f64()));
                } else {
                    return Err("- requires numeric arguments".into());
                }
            }

            let mut _result = 0.0;
            if let Value::Number(num) = &args[0] {
                _result = num.as_f64();
            } else {
                return Err("- requires numeric arguments".into());
            }

            for arg in &args[1..] {
                if let Value::Number(num) = arg {
                    _result -= num.as_f64();
                } else {
                    return Err("- requires numeric arguments".into());
                }
            }

            Ok(Value::from(_result))
        })),
    );

    env.insert(
        "*".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            let mut product = 1.0;
            for arg in args {
                if let Value::Number(num) = arg {
                    product *= num.as_f64();
                } else {
                    return Err("* requires numeric arguments".into());
                }
            }
            Ok(Value::from(product))
        })),
    );

    env.insert(
        "/".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.is_empty() {
                return Err("/ requires at least one argument".into());
            }

            if args.len() == 1 {
                if let Value::Number(num) = &args[0] {
                    let value = num.as_f64();
                    if value == 0.0 {
                        return Err("Division by zero".into());
                    }
                    return Ok(Value::from(1.0 / value));
                } else {
                    return Err("/ requires numeric arguments".into());
                }
            }

            let mut _result = 0.0;
            if let Value::Number(num) = &args[0] {
                _result = num.as_f64();
            } else {
                return Err("/ requires numeric arguments".into());
            }

            for arg in &args[1..] {
                if let Value::Number(num) = arg {
                    let value = num.as_f64();
                    if value == 0.0 {
                        return Err("Division by zero".into());
                    }
                    _result /= value;
                } else {
                    return Err("/ requires numeric arguments".into());
                }
            }

            Ok(Value::from(_result))
        })),
    );

    // Comparison operations
    env.insert(
        "=".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("= requires at least two arguments".into());
            }

            if let Value::Number(first) = &args[0] {
                let first_val = first.as_f64();
                for arg in &args[1..] {
                    if let Value::Number(num) = arg {
                        if first_val != num.as_f64() {
                            return Ok(Value::Boolean(false));
                        }
                    } else {
                        return Err("= requires numeric arguments".into());
                    }
                }
                Ok(Value::Boolean(true))
            } else {
                Err("= requires numeric arguments".into())
            }
        })),
    );

    env.insert(
        "<".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("< requires at least two arguments".into());
            }

            let mut _prev = 0.0;
            if let Value::Number(num) = &args[0] {
                _prev = num.as_f64();
            } else {
                return Err("< requires numeric arguments".into());
            }

            for arg in &args[1..] {
                if let Value::Number(num) = arg {
                    let curr = num.as_f64();
                    if _prev >= curr {
                        return Ok(Value::Boolean(false));
                    }
                    _prev = curr;
                } else {
                    return Err("< requires numeric arguments".into());
                }
            }

            Ok(Value::Boolean(true))
        })),
    );

    env.insert(
        ">".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("> requires at least two arguments".into());
            }

            let mut _prev = 0.0;
            if let Value::Number(num) = &args[0] {
                _prev = num.as_f64();
            } else {
                return Err("> requires numeric arguments".into());
            }

            for arg in &args[1..] {
                if let Value::Number(num) = arg {
                    let curr = num.as_f64();
                    if _prev <= curr {
                        return Ok(Value::Boolean(false));
                    }
                    _prev = curr;
                } else {
                    return Err("> requires numeric arguments".into());
                }
            }

            Ok(Value::Boolean(true))
        })),
    );

    env.insert(
        "<=".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("<= requires at least two arguments".into());
            }

            let mut _prev = 0.0;
            if let Value::Number(num) = &args[0] {
                _prev = num.as_f64();
            } else {
                return Err("<= requires numeric arguments".into());
            }

            for arg in &args[1..] {
                if let Value::Number(num) = arg {
                    let curr = num.as_f64();
                    if _prev > curr {
                        return Ok(Value::Boolean(false));
                    }
                    _prev = curr;
                } else {
                    return Err("<= requires numeric arguments".into());
                }
            }

            Ok(Value::Boolean(true))
        })),
    );

    env.insert(
        ">=".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err(">= requires at least two arguments".into());
            }

            let mut _prev = 0.0;
            if let Value::Number(num) = &args[0] {
                _prev = num.as_f64();
            } else {
                return Err(">= requires numeric arguments".into());
            }

            for arg in &args[1..] {
                if let Value::Number(num) = arg {
                    let curr = num.as_f64();
                    if _prev < curr {
                        return Ok(Value::Boolean(false));
                    }
                    _prev = curr;
                } else {
                    return Err(">= requires numeric arguments".into());
                }
            }

            Ok(Value::Boolean(true))
        })),
    );

    // Boolean operations
    env.insert(
        "not".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("not requires exactly one argument".into());
            }
            match &args[0] {
                Value::Boolean(false) => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            }
        })),
    );

    // Pair and list operations
    env.insert(
        "cons".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 2 {
                return Err("cons requires exactly 2 arguments".into());
            }
            Ok(Value::Pair(Rc::new((args[0].clone(), args[1].clone()))))
        })),
    );

    env.insert(
        "car".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("car requires exactly 1 argument".into());
            }
            if let Value::Pair(pair) = &args[0] {
                Ok(pair.0.clone())
            } else {
                Err("car requires a pair".into())
            }
        })),
    );

    env.insert(
        "cdr".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("cdr requires exactly 1 argument".into());
            }
            if let Value::Pair(pair) = &args[0] {
                Ok(pair.1.clone())
            } else {
                Err("cdr requires a pair".into())
            }
        })),
    );

    // Type predicates
    env.insert(
        "pair?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("pair? requires exactly 1 argument".into());
            }
            Ok(Value::Boolean(matches!(args[0], Value::Pair(_))))
        })),
    );

    env.insert(
        "null?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("null? requires exactly 1 argument".into());
            }
            Ok(Value::Boolean(matches!(args[0], Value::Nil)))
        })),
    );

    env.insert(
        "boolean?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("boolean? requires exactly 1 argument".into());
            }
            Ok(Value::Boolean(matches!(args[0], Value::Boolean(_))))
        })),
    );

    env.insert(
        "symbol?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("symbol? requires exactly 1 argument".into());
            }
            Ok(Value::Boolean(matches!(args[0], Value::Symbol(_))))
        })),
    );

    env.insert(
        "number?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("number? requires exactly 1 argument".into());
            }
            Ok(Value::Boolean(matches!(args[0], Value::Number(_))))
        })),
    );

    env.insert(
        "string?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("string? requires exactly 1 argument".into());
            }
            Ok(Value::Boolean(matches!(args[0], Value::String(_))))
        })),
    );

    env.insert(
        "procedure?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("procedure? requires exactly 1 argument".into());
            }
            Ok(Value::Boolean(matches!(args[0], Value::Procedure(_))))
        })),
    );

    env.insert(
        "char?".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("char? requires exactly 1 argument".into());
            }
            Ok(Value::Boolean(matches!(args[0], Value::Character(_))))
        })),
    );

    // List operations
    env.insert(
        "list".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            let mut result = Value::Nil;
            for arg in args.iter().rev() {
                result = Value::Pair(Rc::new((arg.clone(), result)));
            }
            Ok(result)
        })),
    );

    env.insert(
        "length".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("length requires exactly 1 argument".into());
            }

            let mut count = 0;
            let mut current = args[0].clone();

            while let Value::Pair(pair) = current {
                count += 1;
                current = pair.1.clone();
            }

            if !matches!(current, Value::Nil) {
                return Err("length requires a proper list".into());
            }

            Ok(Value::Number(NumberKind::Integer(count)))
        })),
    );

    // String operations
    env.insert(
        "string-append".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            let mut result = String::new();
            for arg in args {
                if let Value::String(s) = arg {
                    result.push_str(&s);
                } else {
                    return Err("string-append requires string arguments".into());
                }
            }
            Ok(Value::String(result))
        })),
    );

    env.insert(
        "string-length".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("string-length requires exactly 1 argument".into());
            }

            if let Value::String(s) = &args[0] {
                Ok(Value::Number(NumberKind::Integer(s.len() as i64)))
            } else {
                Err("string-length requires a string argument".into())
            }
        })),
    );

    // Conversion procedures
    env.insert(
        "number->string".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("number->string requires exactly 1 argument".into());
            }

            if let Value::Number(num) = &args[0] {
                match num {
                    NumberKind::Integer(i) => Ok(Value::String(i.to_string())),
                    NumberKind::Real(r) => Ok(Value::String(r.to_string())),
                    NumberKind::Rational(n, d) => Ok(Value::String(format!("{}/{}", n, d))),
                }
            } else {
                Err("number->string requires a number argument".into())
            }
        })),
    );

    env.insert(
        "string->number".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("string->number requires exactly 1 argument".into());
            }

            if let Value::String(s) = &args[0] {
                if let Ok(n) = s.parse::<i64>() {
                    Ok(Value::Number(NumberKind::Integer(n)))
                } else if let Ok(f) = s.parse::<f64>() {
                    Ok(Value::Number(NumberKind::Real(f)))
                } else {
                    Ok(Value::Boolean(false))
                }
            } else {
                Err("string->number requires a string argument".into())
            }
        })),
    );

    env.insert(
        "symbol->string".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("symbol->string requires exactly 1 argument".into());
            }

            if let Value::Symbol(s) = &args[0] {
                Ok(Value::String(s.clone()))
            } else {
                Err("symbol->string requires a symbol argument".into())
            }
        })),
    );

    env.insert(
        "string->symbol".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("string->symbol requires exactly 1 argument".into());
            }

            if let Value::String(s) = &args[0] {
                Ok(Value::Symbol(s.clone()))
            } else {
                Err("string->symbol requires a string argument".into())
            }
        })),
    );

    // IO procedures
    env.insert(
        "display".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() != 1 {
                return Err("display requires exactly 1 argument".into());
            }

            match &args[0] {
                Value::String(s) => print!("{}", s),
                other => print!("{}", other),
            }

            Ok(Value::Nil)
        })),
    );

    env.insert(
        "newline".to_string(),
        Value::Procedure(Rc::new(|_args: Vec<Value>| {
            println!();
            Ok(Value::Nil)
        })),
    );

    // Advanced list operations
    env.insert(
        "append".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.is_empty() {
                return Ok(Value::Nil);
            }

            // Handle all but the last argument
            let result = if args.len() == 1 {
                args[0].clone()
            } else {
                let mut result = args.last().unwrap().clone();

                for arg in args[0..args.len() - 1].iter().rev() {
                    let mut current = arg.clone();
                    let mut elements = Vec::new();

                    // Extract elements from the list
                    while let Value::Pair(pair) = current.clone() {
                        elements.push(pair.0.clone());
                        current = pair.1.clone();
                    }

                    // Check if it's a proper list
                    if !matches!(current, Value::Nil) {
                        return Err("append requires proper lists".into());
                    }

                    // Append elements to the result
                    for element in elements.iter().rev() {
                        result = Value::Pair(Rc::new((element.clone(), result)));
                    }
                }

                result
            };

            Ok(result)
        })),
    );

    // Function composition
    env.insert(
        "apply".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("apply requires at least 2 arguments".into());
            }

            let proc = args[0].clone();
            let last_arg = args.last().unwrap();

            let mut apply_args = Vec::new();

            // Add all arguments except the last and the procedure
            for arg in &args[1..args.len() - 1] {
                apply_args.push(arg.clone());
            }

            // Add elements from the last argument (which should be a list)
            let mut current = last_arg.clone();
            while let Value::Pair(pair) = current.clone() {
                apply_args.push(pair.0.clone());
                current = pair.1.clone();
            }

            // Ensure the last argument is a proper list
            if !matches!(current, Value::Nil) {
                return Err("The last argument to apply must be a proper list".into());
            }

            // Apply the procedure
            match proc {
                Value::Procedure(p) => p(apply_args),
                _ => Err("First argument to apply must be a procedure".into()),
            }
        })),
    );

    env.insert(
        "map".to_string(),
        Value::Procedure(Rc::new(|args: Vec<Value>| {
            if args.len() < 2 {
                return Err("map requires at least 2 arguments".into());
            }

            let proc = args[0].clone();
            let mut lists = Vec::new();

            // Validate all arguments are lists and have the same length
            for arg in &args[1..] {
                let mut current = arg.clone();
                let mut list = Vec::new();

                while let Value::Pair(pair) = current.clone() {
                    list.push(pair.0.clone());
                    current = pair.1.clone();
                }

                // Ensure it's a proper list
                if !matches!(current, Value::Nil) {
                    return Err("map requires proper lists".into());
                }

                lists.push(list);
            }

            // Ensure all lists have the same length
            let len = lists[0].len();
            for list in &lists {
                if list.len() != len {
                    return Err("All lists passed to map must have the same length".into());
                }
            }

            // Apply the procedure to each set of elements
            let mut results = Vec::new();
            for i in 0..len {
                let mut proc_args = Vec::new();
                for list in &lists {
                    proc_args.push(list[i].clone());
                }

                match proc.clone() {
                    Value::Procedure(p) => {
                        let result = p(proc_args)?;
                        results.push(result);
                    }
                    _ => return Err("First argument to map must be a procedure".into()),
                }
            }

            // Construct the result list
            let mut result = Value::Nil;
            for res in results.iter().rev() {
                result = Value::Pair(Rc::new((res.clone(), result)));
            }

            Ok(result)
        })),
    );
}
