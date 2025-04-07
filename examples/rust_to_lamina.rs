use lamina::embed::{self, types::*};
use lamina::value::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust to Lamina Example ===");

    // Initialize the Lamina interpreter
    let interpreter = embed::init();

    // Evaluate some Lamina code
    let result = interpreter.eval("(+ 1 2 3)")?;
    println!("Evaluated (+ 1 2 3) => {}", result);

    // Define a variable in Lamina
    interpreter.define("my-var", Value::from(42));

    // Retrieve a variable from Lamina
    let value = interpreter.get("my-var").unwrap();
    println!("Retrieved my-var => {}", value);

    // Call a Lamina procedure from Rust
    let args = vec![Value::from(5), Value::from(7)];
    let result = interpreter.call("+", args)?;
    println!("Called (+ 5 7) => {}", result);

    // Register a Rust function in Lamina
    interpreter.register_function("rust-function", |args| {
        if args.len() != 2 {
            return Err("rust-function requires 2 arguments".into());
        }

        let arg1 = value_to_i64(&args[0])?;
        let arg2 = value_to_i64(&args[1])?;

        Ok(i64_to_value(arg1 * arg2))
    });

    // Call the Rust function from Lamina
    let result = interpreter.eval("(rust-function 6 7)")?;
    println!("Called (rust-function 6 7) => {}", result);

    // Evaluate more complex Lamina code that uses our Rust function
    let code = r#"
    (define (square x) (* x x))
    (define (use-rust a b) (rust-function a b))
    (use-rust (square 4) 5)
    "#;

    let result = interpreter.eval(code)?;
    println!("Evaluated complex code => {}", result);

    Ok(())
}
