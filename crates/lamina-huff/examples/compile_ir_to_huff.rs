use lamina_huff::{compile_to_huff, HuffOptions};
use lamina_ir::ir::{BinOp, Def, Expr, Ident, Program, Type};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lamina IR to Huff Compiler Example ===");

    // Create a simple counter program using Lamina IR
    let mut program = Program::new();

    // Add metadata
    program.add_metadata("name", "Counter");
    program.add_metadata("author", "Lamina Team");

    // Define constants
    let counter_slot = Def::Const {
        name: Ident("COUNTER_SLOT".to_string()),
        ty: Type::Uint(256),
        value: Expr::UintLit(0),
    };
    program.add_def(counter_slot);

    // Define get-counter function
    let get_counter = Def::Function {
        name: Ident("get_counter".to_string()),
        params: vec![],
        return_type: Type::Uint(256),
        body: Expr::Call(
            Box::new(Expr::Var(Ident("storage-load".to_string()))),
            vec![Expr::Var(Ident("COUNTER_SLOT".to_string()))],
        ),
    };
    program.add_def(get_counter);

    // Define increment function
    let increment = Def::Function {
        name: Ident("increment".to_string()),
        params: vec![],
        return_type: Type::Uint(256),
        body: Expr::Let(
            Ident("current".to_string()),
            Box::new(Expr::Call(
                Box::new(Expr::Var(Ident("storage-load".to_string()))),
                vec![Expr::Var(Ident("COUNTER_SLOT".to_string()))],
            )),
            Box::new(Expr::Let(
                Ident("_".to_string()),
                Box::new(Expr::Call(
                    Box::new(Expr::Var(Ident("storage-store".to_string()))),
                    vec![
                        Expr::Var(Ident("COUNTER_SLOT".to_string())),
                        Expr::BinOp(
                            BinOp::Add,
                            Box::new(Expr::Var(Ident("current".to_string()))),
                            Box::new(Expr::UintLit(1)),
                        ),
                    ],
                )),
                Box::new(Expr::Call(
                    Box::new(Expr::Var(Ident("storage-load".to_string()))),
                    vec![Expr::Var(Ident("COUNTER_SLOT".to_string()))],
                )),
            )),
        ),
    };
    program.add_def(increment);

    // Compile to Huff
    println!("Compiling IR to Huff...");
    let options = HuffOptions {
        output_dir: "examples/output".to_string(),
        base_name: "CounterFromIR".to_string(),
        optimize: true,
    };

    // Create the output directory if it doesn't exist
    let output_dir = Path::new(&options.output_dir);
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // Compile from IR to Huff
    let huff_code = compile_to_huff(&program, &options)?;

    // Write to a file
    let output_path = output_dir.join(format!("{}.huff", options.base_name));
    println!("Writing Huff code to: {}", output_path.display());
    fs::write(&output_path, &huff_code)?;

    println!("Compilation successful!");
    println!(
        "Generated Huff code has been written to {}",
        output_path.display()
    );

    // Show the Huff code
    println!("\nGenerated Huff Code:");
    println!("====================");
    println!("{}", huff_code);

    Ok(())
}
