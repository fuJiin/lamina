use lamina::evaluator;
use lamina::evaluator::environment::setup_initial_env;
use lamina::ffi::{self, rustlib};
use lamina::lexer;
use lamina::parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust Module Example ===");

    // Create and register a "math" module
    rustlib::create_module("math", |module| {
        // Add some math functions
        module.add_function("add", |args| {
            if args.len() != 2 {
                return Err("math/add requires 2 arguments".into());
            }

            let x = ffi::value_to_f64(&args[0])?;
            let y = ffi::value_to_f64(&args[1])?;

            Ok(ffi::f64_to_value(x + y))
        });

        module.add_function("subtract", |args| {
            if args.len() != 2 {
                return Err("math/subtract requires 2 arguments".into());
            }

            let x = ffi::value_to_f64(&args[0])?;
            let y = ffi::value_to_f64(&args[1])?;

            Ok(ffi::f64_to_value(x - y))
        });

        module.add_function("square", |args| {
            if args.len() != 1 {
                return Err("math/square requires 1 argument".into());
            }

            let x = ffi::value_to_f64(&args[0])?;

            Ok(ffi::f64_to_value(x * x))
        });
    });

    // Create and register a "string" module
    rustlib::create_module("string", |module| {
        module.add_function("length", |args| {
            if args.len() != 1 {
                return Err("string/length requires 1 argument".into());
            }

            let s = ffi::value_to_string(&args[0])?;

            Ok(ffi::i64_to_value(s.len() as i64))
        });

        module.add_function("uppercase", |args| {
            if args.len() != 1 {
                return Err("string/uppercase requires 1 argument".into());
            }

            let s = ffi::value_to_string(&args[0])?;

            Ok(ffi::string_to_value(s.to_uppercase()))
        });
    });

    // Set up a Lamina environment
    let env = setup_initial_env();

    // Import our Rust modules
    rustlib::import_module("math", &env).unwrap();
    rustlib::import_module("string", &env).unwrap();

    // Use the modules from Lamina
    let code = r#"
    (begin
      (define x 5)
      (define y 3)
      
      (display "Math operations:\n")
      (display "  Square of 5: ")
      (display (math/square x))
      (newline)
      
      (display "  5 + 3 = ")
      (display (math/add x y))
      (newline)
      
      (display "  5 - 3 = ")
      (display (math/subtract x y))
      (newline)
      
      (display "\nString operations:\n")
      (define message "Hello, Lamina!")
      (display "  Original: ")
      (display message)
      (newline)
      
      (display "  Length: ")
      (display (string/length message))
      (newline)
      
      (display "  Uppercase: ")
      (display (string/uppercase message))
      (newline)
      
      (list (math/square x) (string/uppercase message)))
    "#;

    let tokens = lexer::lex(code)?;
    let expr = parser::parse(&tokens)?;
    let result = evaluator::eval_with_env(expr, env)?;

    println!("\nFinal result: {}", result);

    Ok(())
}
