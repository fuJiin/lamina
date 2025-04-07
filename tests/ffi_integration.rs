use lamina::embed;
use lamina::ffi;
use lamina::value::{Value, NumberKind};

#[test]
fn test_bidirectional_ffi() {
    // Create a Lamina interpreter
    let interpreter = embed::init();
    
    // 1. Register a Rust function
    interpreter.register_function("test-multiply", |args| {
        if args.len() != 2 {
            return Err("test-multiply requires 2 arguments".into());
        }
        
        let arg1 = ffi::value_to_f64(&args[0])?;
        let arg2 = ffi::value_to_f64(&args[1])?;
        
        Ok(ffi::f64_to_value(arg1 * arg2))
    });
    
    // 2. Test the Rust function directly
    let result = interpreter.eval("(test-multiply 5 6)").unwrap();
    
    if let Value::Number(NumberKind::Real(n)) = result {
        assert_eq!(n, 30.0);
    } else {
        panic!("Expected Real number result, got: {:?}", result);
    }
}

#[test]
fn test_rust_module() {
    // Create a Lamina interpreter
    let interpreter = embed::init();
    
    // Create and register a test module
    ffi::rustlib::create_module("test-module", |module| {
        module.add_function("add", |args| {
            if args.len() != 2 {
                return Err("test-module/add requires 2 arguments".into());
            }
            
            let x = ffi::value_to_f64(&args[0])?;
            let y = ffi::value_to_f64(&args[1])?;
            
            Ok(ffi::f64_to_value(x + y))
        });
    });
    
    // Import the module
    ffi::rustlib::import_module("test-module", &interpreter.environment()).unwrap();
    
    // Test the module function
    let result = interpreter.eval("(test-module/add 10 20)").unwrap();
    
    if let Value::Number(NumberKind::Real(n)) = result {
        assert_eq!(n, 30.0);
    } else {
        panic!("Expected Real number result, got: {:?}", result);
    }
} 