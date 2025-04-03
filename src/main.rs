
mod lexer;
mod parser;
mod evaluator;
mod value;
mod error;

use rustyline::Editor;
use std::fs;
use value::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        let filename = &args[1];
        if !filename.ends_with(".lmn") {
            eprintln!("Error: File must have .lmn extension");
            std::process::exit(1);
        }
        let content = fs::read_to_string(filename)?;
        execute(&content)?;
    } else {
        repl()?;
    }
    Ok(())
}

fn execute(source: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let tokens = lexer::lex(source)?;
    let ast = parser::parse(tokens)?;
    evaluator::eval(ast)
}

fn repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = Editor::<()>::new()?;
    println!("Lamina R7RS-small (Press Ctrl+C to exit)");
    
    loop {
        match rl.readline("λ> ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                match execute(&line) {
                    Ok(val) => println!("{}", val),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}
