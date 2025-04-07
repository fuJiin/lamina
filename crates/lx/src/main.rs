use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Lamina project
    New {
        /// Name of the project
        name: String,
    },
    /// Initialize a Lamina project in the current directory
    Init {},
    /// Build the Lamina project
    Build {
        /// Optional target backend (default: interpreter)
        #[arg(short, long)]
        target: Option<String>,
    },
    /// Run a Lamina script
    Run {
        /// Path to the script
        script: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => {
            println!("Creating new project: {}", name);
            // TODO: Implement project creation
        }
        Commands::Init {} => {
            println!("Initializing project in current directory");
            // TODO: Implement project initialization
        }
        Commands::Build { target } => {
            match target {
                Some(t) => println!("Building project with target: {}", t),
                None => println!("Building project with default target"),
            }
            // TODO: Implement build
        }
        Commands::Run { script } => {
            println!("Running script: {:?}", script);
            // TODO: Implement script running
        }
    }
} 