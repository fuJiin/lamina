use lamina_huff::huff;
use lamina::evaluator;
use lamina::evaluator::environment::setup_initial_env;
use lamina::lexer;
use lamina::parser;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lamina to Huff Compiler Example ===");

    // Simple Lamina code for a counter contract with automatic function dispatch
    let lamina_code = r#"
    (begin
      ;; Define a storage slot for our counter
      (define counter-slot 0)
      
      ;; Get the counter value
      (define (get-counter)
        (storage-load counter-slot))
      
      ;; Increment the counter
      (define (increment)
        (begin
          (define current (storage-load counter-slot))
          (storage-store counter-slot (+ current 1))
          (storage-load counter-slot)))
    )"#;

    println!("Parsing Lamina code...");

    // Lexer and parse the code
    let tokens = lexer::lex(lamina_code)?;
    let expr = parser::parse(&tokens)?;

    // Create an environment
    let env = setup_initial_env();

    // Evaluate the code to ensure it's valid
    println!("Validating Lamina code...");
    let _ = evaluator::eval_with_env(expr.clone(), env)?;

    // Compile to Huff
    println!("Compiling to Huff...");
    let huff_code = huff::compile(&expr, "Counter")?;

    // Output directory
    let output_dir = Path::new("examples/output");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // Write to a file
    let output_path = output_dir.join("Counter.huff");
    println!("Writing Huff code to: {}", output_path.display());
    fs::write(&output_path, huff_code)?;

    println!("Compilation successful!");
    println!(
        "Generated Huff code has been written to {}",
        output_path.display()
    );

    println!("\nNotice: The compiler now automatically generates function selectors!");
    println!("You no longer need to hardcode selectors in your Lamina code.");

    Ok(())
}
