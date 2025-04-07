use lamina::evaluator;
use lamina::evaluator::environment::setup_initial_env;
use lamina::ffi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lamina to Rust Example ===");

    // Register some Rust functions that Lamina can call
    ffi::register_function("rust-multiply", |args| {
        if args.len() != 2 {
            return Err("rust-multiply requires 2 arguments".into());
        }

        let arg1 = ffi::value_to_f64(&args[0])?;
        let arg2 = ffi::value_to_f64(&args[1])?;

        Ok(ffi::f64_to_value(arg1 * arg2))
    });

    ffi::register_function("rust-string-length", |args| {
        if args.len() != 1 {
            return Err("rust-string-length requires 1 argument".into());
        }

        let s = ffi::value_to_string(&args[0])?;
        Ok(ffi::i64_to_value(s.len() as i64))
    });

    // A more complex Rust function
    ffi::register_function("rust-http-user-agent", |_args| {
        // In a real implementation, this would make an actual HTTP request
        // For demonstration purposes, we'll just return a mock response
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        Ok(ffi::string_to_value(user_agent.to_string()))
    });

    // Create a Lamina environment
    let env = setup_initial_env();

    // Run Lamina code that calls our Rust functions
    let code = r#"
    (begin
      (define result-1 (rust-multiply 3.5 2.5))
      (define result-2 (rust-string-length "Hello, Rust!"))
      (define user-agent (rust-http-user-agent))
      
      (display "Result 1: ")
      (display result-1)
      (newline)
      
      (display "Result 2: ")
      (display result-2)
      (newline)
      
      (display "User Agent: ")
      (display user-agent)
      (newline)
      
      (list result-1 result-2 user-agent))
    "#;

    let tokens = lamina::lexer::lex(code)?;
    let expr = lamina::parser::parse(&tokens)?;
    let result = evaluator::eval_with_env(expr, env)?;

    println!("\nFinal result: {}", result);

    Ok(())
}
