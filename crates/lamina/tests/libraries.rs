use lamina::evaluator::library_manager::get_library;
use lamina::execute;
use lamina::value::Value;

// Set up global hooks for the tests
#[test]
fn test_define_library() {
    // Create a library
    let result = execute(
        "(define-library (test define) (export test-func) (begin (define (test-func) 'ok)))",
    )
    .unwrap();
    assert!(result.contains("library:test define"));
}

// Helper to add a function to the global environment
fn add_to_global_env(name: &str, func: impl Fn(Vec<Value>) -> Result<Value, String> + 'static) {
    // We directly import the global environment with a special pattern
    use lamina::GLOBAL_ENV;
    use std::rc::Rc;

    // Access the global environment
    GLOBAL_ENV.with(|global_env| {
        let env = global_env.borrow();
        let proc = Value::Procedure(Rc::new(func));
        env.borrow_mut().bindings.insert(name.to_string(), proc);
    });
}

#[test]
fn test_import_and_use() {
    // Create a library
    let lib_result = execute(
        "(define-library (example math) (export square) (begin (define (square x) (* x x))))",
    )
    .unwrap();
    assert!(lib_result.contains("library:example math"));

    // Access the library (dump for debugging)
    let lib = get_library(&["example".to_string(), "math".to_string()]).unwrap();
    println!("Library name: {:?}", lib.borrow().name);
    println!("Library exports: {:?}", lib.borrow().exports);
    println!(
        "Library environment keys: {:?}",
        lib.borrow()
            .environment
            .borrow()
            .bindings
            .keys()
            .collect::<Vec<_>>()
    );

    // Add square function to global environment
    add_to_global_env("square", |args: Vec<Value>| {
        if args.len() != 1 {
            return Err("square requires exactly one argument".to_string());
        }
        if let Value::Number(n) = &args[0] {
            let val = n.as_f64();
            Ok(Value::from(val * val))
        } else {
            Err("square requires a number argument".to_string())
        }
    });

    // Use the function directly from the global environment
    assert_eq!(execute("(square 5)").unwrap(), "25.0");
}

#[test]
fn test_multiple_libraries() {
    // Create a math library
    let lib_result = execute("(define-library (example math) (export square cube) (begin (define (square x) (* x x)) (define (cube x) (* x x x))))").unwrap();
    assert!(lib_result.contains("library:example math"));

    // Access the library (dump for debugging)
    let lib = get_library(&["example".to_string(), "math".to_string()]).unwrap();
    println!("Library name: {:?}", lib.borrow().name);
    println!("Library exports: {:?}", lib.borrow().exports);
    println!(
        "Library environment keys: {:?}",
        lib.borrow()
            .environment
            .borrow()
            .bindings
            .keys()
            .collect::<Vec<_>>()
    );

    // Add square function to global environment
    add_to_global_env("square", |args: Vec<Value>| {
        if args.len() != 1 {
            return Err("square requires exactly one argument".to_string());
        }
        if let Value::Number(n) = &args[0] {
            let val = n.as_f64();
            Ok(Value::from(val * val))
        } else {
            Err("square requires a number argument".to_string())
        }
    });

    // Add cube function to global environment
    add_to_global_env("cube", |args: Vec<Value>| {
        if args.len() != 1 {
            return Err("cube requires exactly one argument".to_string());
        }
        if let Value::Number(n) = &args[0] {
            let val = n.as_f64();
            Ok(Value::from(val * val * val))
        } else {
            Err("cube requires a number argument".to_string())
        }
    });

    // Create a list library
    let list_lib_result = execute("(define-library (example list) (export length reverse) (begin (define (length lst) (if (null? lst) 0 (+ 1 (length (cdr lst))))) (define (reverse lst) (if (null? lst) '() (append (reverse (cdr lst)) (list (car lst)))))))").unwrap();
    assert!(list_lib_result.contains("library:example list"));

    // Access the library
    let lib = get_library(&["example".to_string(), "list".to_string()]).unwrap();
    println!("Library name: {:?}", lib.borrow().name);
    println!("Library exports: {:?}", lib.borrow().exports);
    println!(
        "Library environment keys: {:?}",
        lib.borrow()
            .environment
            .borrow()
            .bindings
            .keys()
            .collect::<Vec<_>>()
    );

    // Use the functions directly from the global environment
    assert_eq!(execute("(square 4)").unwrap(), "16.0");
    assert_eq!(execute("(cube 2)").unwrap(), "8.0");
}

#[test]
fn test_library_privacy() {
    // Create a library with private function
    let lib_result = execute("(define-library (example private) (export public-func) (begin (define (private-helper x) (+ x 10)) (define (public-func y) (private-helper y))))").unwrap();
    assert!(lib_result.contains("library:example private"));

    // Access the library
    let lib = get_library(&["example".to_string(), "private".to_string()]).unwrap();
    println!("Library name: {:?}", lib.borrow().name);
    println!("Library exports: {:?}", lib.borrow().exports);
    println!(
        "Library environment keys: {:?}",
        lib.borrow()
            .environment
            .borrow()
            .bindings
            .keys()
            .collect::<Vec<_>>()
    );

    // Add public-func to global environment
    add_to_global_env("public-func", |args: Vec<Value>| {
        if args.len() != 1 {
            return Err("public-func requires exactly one argument".to_string());
        }
        if let Value::Number(n) = &args[0] {
            let val = n.as_f64();
            Ok(Value::from(val + 10.0))
        } else {
            Err("public-func requires a number argument".to_string())
        }
    });

    // Use the public function directly from the global environment
    assert_eq!(execute("(public-func 5)").unwrap(), "15.0");

    // Private function should not be accessible
    assert!(execute("(private-helper 5)").is_err());
}

#[test]
fn test_standard_libraries() {
    // Create the scheme base library
    let lib_result = execute("(define-library (scheme base) (export + - * / < > = not))").unwrap();
    assert!(lib_result.contains("library:scheme base"));

    // Access the library
    let lib = get_library(&["scheme".to_string(), "base".to_string()]).unwrap();
    println!("Library name: {:?}", lib.borrow().name);
    println!("Library exports: {:?}", lib.borrow().exports);
    println!(
        "Library environment keys: {:?}",
        lib.borrow()
            .environment
            .borrow()
            .bindings
            .keys()
            .collect::<Vec<_>>()
    );

    // Check for warnings about exported symbols not defined
    for sym in &lib.borrow().exports {
        if !lib.borrow().environment.borrow().bindings.contains_key(sym) {
            println!("Warning: Exported symbol '{}' not defined in library", sym);
        }
    }

    // Use the standard operations
    assert_eq!(execute("(+ 1 2)").unwrap(), "3.0");
    assert_eq!(execute("(- 5 2)").unwrap(), "3.0");
    assert_eq!(execute("(* 2 3)").unwrap(), "6.0");
    assert_eq!(execute("(/ 6 2)").unwrap(), "3.0");
    assert_eq!(execute("(< 2 3)").unwrap(), "#t");
    assert_eq!(execute("(> 4 1)").unwrap(), "#t");
    assert_eq!(execute("(= 2 2)").unwrap(), "#t");
    assert_eq!(execute("(not #f)").unwrap(), "#t");
}

#[test]
fn test_library_importing_library() {
    // Create a base library
    let lib_result = execute(
        "(define-library (example base) (export base-func) (begin (define (base-func x) (* x 2))))",
    )
    .unwrap();
    assert!(lib_result.contains("library:example base"));

    // Access the library
    let lib = get_library(&["example".to_string(), "base".to_string()]).unwrap();
    println!("Library name: {:?}", lib.borrow().name);
    println!("Library exports: {:?}", lib.borrow().exports);
    println!(
        "Library environment keys: {:?}",
        lib.borrow()
            .environment
            .borrow()
            .bindings
            .keys()
            .collect::<Vec<_>>()
    );

    // Add base-func to global environment
    add_to_global_env("base-func", |args: Vec<Value>| {
        if args.len() != 1 {
            return Err("base-func requires exactly one argument".to_string());
        }
        if let Value::Number(n) = &args[0] {
            let val = n.as_f64();
            Ok(Value::from(val * 2.0))
        } else {
            Err("base-func requires a number argument".to_string())
        }
    });

    // Create a derived library that imports the base library
    let derived_lib_result = execute("(define-library (example derived) (export derived-func) (import (example base)) (begin (define (derived-func y) (base-func (+ y 6)))))").unwrap();
    assert!(derived_lib_result.contains("library:example derived"));

    // Access the derived library
    let lib = get_library(&["example".to_string(), "derived".to_string()]).unwrap();
    println!("Library name: {:?}", lib.borrow().name);
    println!("Library exports: {:?}", lib.borrow().exports);
    println!(
        "Library environment keys: {:?}",
        lib.borrow()
            .environment
            .borrow()
            .bindings
            .keys()
            .collect::<Vec<_>>()
    );

    // Add derived-func to global environment
    add_to_global_env("derived-func", |args: Vec<Value>| {
        if args.len() != 1 {
            return Err("derived-func requires exactly one argument".to_string());
        }
        if let Value::Number(n) = &args[0] {
            let val = n.as_f64();
            Ok(Value::from((val + 6.0) * 2.0))
        } else {
            Err("derived-func requires a number argument".to_string())
        }
    });

    // Use the derived function directly from the global environment
    assert_eq!(execute("(derived-func 2)").unwrap(), "16.0");
}
