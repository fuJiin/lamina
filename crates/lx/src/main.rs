use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a Lamina file to run
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
    
    /// Arguments to pass to the script
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Lamina project
    New {
        /// Name of the project
        name: String,
        
        /// Target backend (default: native)
        #[arg(short, long, default_value = "native")]
        target: String,
    },
    /// Initialize a Lamina project in the current directory
    Init {
        /// Target backend (default: native)
        #[arg(short, long, default_value = "native")]
        target: String,
    },
    /// Build the Lamina project
    Build {
        /// Target backend (native, evm, etc.)
        #[arg(short, long, default_value = "native")]
        target: String,
        
        /// Optimization level (0-3)
        #[arg(short, long, default_value_t = 0)]
        opt_level: u8,
    },
    /// Run a Lamina script
    Run {
        /// Path to the script
        script: PathBuf,
        
        /// Arguments to pass to the script
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Start the Lamina REPL
    Repl {},
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::New { name, target }) => {
            println!("Creating new project: {} with target: {}", name, target);
            // TODO: Implement project creation
        }
        Some(Commands::Init { target }) => {
            println!("Initializing project in current directory with target: {}", target);
            // TODO: Implement project initialization
        }
        Some(Commands::Build { target, opt_level }) => {
            println!("Building project with target: {} at optimization level: {}", target, opt_level);
            
            if target == "native" {
                println!("Using lxc for native compilation");
                // TODO: Invoke lxc here
            } else if target == "evm" {
                println!("Using lamina-huff for EVM compilation");
                // TODO: Invoke lamina-huff here
            } else {
                eprintln!("Unsupported target: {}", target);
            }
        }
        Some(Commands::Run { script, args }) => {
            println!("Running script: {:?} with args: {:?}", script, args);
            // TODO: Implement script running
        }
        Some(Commands::Repl {}) => {
            println!("Starting Lamina REPL...");
            start_repl();
        }
        None => {
            // If a file is provided, run it
            if let Some(file) = &cli.file {
                println!("Running file: {:?} with args: {:?}", file, cli.args);
                // TODO: Implement file running
            } else {
                // No subcommand or file, start REPL
                println!("Starting Lamina REPL...");
                start_repl();
            }
        }
    }
}

/// Start the Lamina REPL
fn start_repl() {
    // This is a placeholder for the actual REPL implementation
    println!("Welcome to Lamina REPL!");
    println!("Type expressions to evaluate them, or :help for more information.");
    
    // In a real implementation, we would:
    // 1. Set up a rustyline editor
    // 2. Parse and evaluate user input
    // 3. Print results
    // 4. Repeat
    
    // For now, just exit
    println!("REPL not yet implemented, exiting...");
} 