use lamina::embed;
use lamina::evaluator;
use lamina::evaluator::environment::setup_initial_env;
use lamina::ffi::{self, rustlib};
use lamina::lexer;
use lamina::parser;
use lamina::value::{NumberKind, Value};
use std::f64::consts::PI;

#[test]
fn test_registering_rust_function() {
    // Create a Lamina interpreter (which automatically loads FFI functions)
    let interpreter = embed::init();

    // Register a simple Rust function
    interpreter.register_function("test-add", |args| {
        if args.len() != 2 {
            return Err("test-add requires 2 arguments".into());
        }

        let arg1 = ffi::value_to_f64(&args[0])?;
        let arg2 = ffi::value_to_f64(&args[1])?;

        Ok(ffi::f64_to_value(arg1 + arg2))
    });

    // Run Lamina code that calls our Rust function
    let result = interpreter.eval("(test-add 3.5 2.5)").unwrap();

    if let Value::Number(NumberKind::Real(n)) = result {
        assert_eq!(n, 6.0);
    } else {
        panic!("Expected Real number result, got: {:?}", result);
    }
}

#[test]
fn test_rust_module() {
    // Create a Lamina environment
    let env = setup_initial_env();

    // Create and register a test module
    rustlib::create_module("test-math", |module| {
        // Add a function
        module.add_function("multiply", |args| {
            if args.len() != 2 {
                return Err("test-math/multiply requires 2 arguments".into());
            }

            let x = ffi::value_to_f64(&args[0])?;
            let y = ffi::value_to_f64(&args[1])?;

            Ok(ffi::f64_to_value(x * y))
        });
    });

    // Import the module
    rustlib::import_module("test-math", &env).unwrap();

    // Use the module from Lamina
    let code = "(test-math/multiply 4 5)";

    let tokens = lexer::lex(code).unwrap();
    let expr = parser::parse(&tokens).unwrap();
    let result = evaluator::eval_with_env(expr, env).unwrap();

    if let Value::Number(NumberKind::Real(n)) = result {
        assert_eq!(n, 20.0);
    } else {
        panic!("Expected Real number result, got: {:?}", result);
    }
}

#[test]
fn test_embedding_api() {
    // Create a Lamina interpreter
    let interpreter = embed::init();

    // Evaluate some Lamina code
    let result = interpreter.eval("(+ 1 2 3)").unwrap();

    if let Value::Number(NumberKind::Real(n)) = result {
        assert_eq!(n, 6.0);
    } else {
        panic!("Expected Real number result, got: {:?}", result);
    }

    // Define a variable in Lamina
    interpreter.define("my-test-var", Value::from(42));

    // Retrieve the variable
    let value = interpreter.get("my-test-var").unwrap();

    if let Value::Number(NumberKind::Integer(n)) = value {
        assert_eq!(n, 42);
    } else {
        panic!("Expected Integer value, got: {:?}", value);
    }

    // Register a Rust function
    interpreter.register_function("test-square", |args| {
        if args.len() != 1 {
            return Err("test-square requires 1 argument".into());
        }

        let arg = ffi::value_to_f64(&args[0])?;

        Ok(ffi::f64_to_value(arg * arg))
    });

    // Call the function
    let result = interpreter.eval("(test-square 5)").unwrap();

    if let Value::Number(NumberKind::Real(n)) = result {
        assert_eq!(n, 25.0);
    } else {
        panic!("Expected Real number result, got: {:?}", result);
    }
}

#[test]
fn test_function_error_handling() {
    // Create a Lamina interpreter
    let interpreter = embed::init();

    // Register a function that validates arguments
    interpreter.register_function("test-divide", |args| {
        if args.len() != 2 {
            return Err("test-divide requires 2 arguments".into());
        }

        let arg1 = ffi::value_to_f64(&args[0])?;
        let arg2 = ffi::value_to_f64(&args[1])?;

        if arg2 == 0.0 {
            return Err("Cannot divide by zero".into());
        }

        Ok(ffi::f64_to_value(arg1 / arg2))
    });

    // Test with valid arguments
    let result = interpreter.eval("(test-divide 10 2)").unwrap();

    if let Value::Number(NumberKind::Real(n)) = result {
        assert_eq!(n, 5.0);
    } else {
        panic!("Expected Real number result, got: {:?}", result);
    }

    // Test with invalid argument count
    let err = interpreter.eval("(test-divide 10)").unwrap_err();
    assert!(err.to_string().contains("test-divide requires 2 arguments"));

    // Test with division by zero
    let err = interpreter.eval("(test-divide 10 0)").unwrap_err();
    assert!(err.to_string().contains("Cannot divide by zero"));
}

#[test]
fn test_type_conversions() {
    // Test converting from Rust to Lamina and back

    // Integer
    let original_int = 42;
    let lamina_int = ffi::i64_to_value(original_int);
    let round_trip_int = ffi::value_to_i64(&lamina_int).unwrap();
    assert_eq!(original_int, round_trip_int);

    // Float
    let original_float = PI;
    let lamina_float = ffi::f64_to_value(original_float);
    let round_trip_float = ffi::value_to_f64(&lamina_float).unwrap();
    assert_eq!(original_float, round_trip_float);

    // Boolean
    let original_bool = true;
    let lamina_bool = ffi::bool_to_value(original_bool);
    let round_trip_bool = ffi::value_to_bool(&lamina_bool).unwrap();
    assert_eq!(original_bool, round_trip_bool);

    // String
    let original_string = "Hello, FFI!".to_string();
    let lamina_string = ffi::string_to_value(original_string.clone());
    let round_trip_string = ffi::value_to_string(&lamina_string).unwrap();
    assert_eq!(original_string, round_trip_string);
}

#[test]
fn test_direct_rustfn_creation() {
    // This test directly constructs a RustFn variant to remove the "dead code" warning
    let rust_fn = ffi::create_rust_fn("test-function", |args: Vec<Value>| {
        Ok(Value::Number(NumberKind::Integer(args.len() as i64)))
    });

    // Verify the type
    match rust_fn {
        Value::RustFn(_, name) => {
            assert_eq!(name, "test-function");
        }
        _ => panic!("Expected RustFn, got: {:?}", rust_fn),
    }
}
