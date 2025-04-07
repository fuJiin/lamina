//! Lamina native compiler
//! 
//! This binary compiles Lamina code to native machine code using rustc's infrastructure.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input Lamina file
    #[arg(value_name = "FILE")]
    input: Option<String>,
    
    /// Output path
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
    
    /// Optimization level (0-3)
    #[arg(short, long, default_value_t = 0)]
    opt_level: u8,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check the code for errors without compiling
    Check { 
        /// Input file
        file: String 
    },
    
    /// Print the IR for the input file
    Ir { 
        /// Input file
        file: String,
        
        /// Whether to print optimized IR
        #[arg(short, long)]
        optimized: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Check { file }) => {
            println!("Checking file: {}", file);
            // TODO: Implement checking
        }
        Some(Commands::Ir { file, optimized }) => {
            println!("Printing {} IR for file: {}", 
                     if optimized { "optimized" } else { "unoptimized" }, 
                     file);
            // TODO: Implement IR printing
        }
        None => {
            if let Some(input) = cli.input {
                println!("Compiling file: {}", input);
                let output = cli.output.unwrap_or_else(|| {
                    // Default to input file stem with appropriate extension
                    let path = std::path::Path::new(&input);
                    let stem = path.file_stem().unwrap().to_str().unwrap();
                    
                    // Output binary
                    #[cfg(target_os = "windows")]
                    let output = format!("{}.exe", stem);
                    #[cfg(not(target_os = "windows"))]
                    let output = stem.to_string();
                    
                    output
                });
                println!("Output: {}", output);
                println!("Optimization level: {}", cli.opt_level);
                // TODO: Implement compilation
            } else {
                println!("No input file specified. Run with --help for usage information.");
            }
        }
    }
} 