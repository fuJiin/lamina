//! Lamina native compiler library
//! 
//! This library provides functionality for compiling Lamina code to native machine code.

use thiserror::Error;

pub mod backend;
pub mod rustc_integration;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("IR error: {0}")]
    IrError(#[from] lamina_ir::IrError),
    
    #[error("Rustc error: {0}")]
    RustcError(String),
    
    #[error("Compilation error: {0}")]
    CompilationError(String),
}

/// Result type for compiler operations
pub type Result<T> = std::result::Result<T, CompilerError>;

/// Struct representing compilation options
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Path to the input file
    pub input: String,
    
    /// Path to the output file
    pub output: String,
    
    /// Optimization level (0-3)
    pub opt_level: u8,
    
    /// Whether to emit debug information
    pub debug_info: bool,
}

/// Compile Lamina code to native machine code
pub fn compile(options: CompileOptions) -> Result<()> {
    // This is a placeholder for the actual implementation
    
    // 1. Parse the input file into Lamina AST
    // 2. Lower the AST to Lamina IR
    // 3. Apply optimizations to the IR
    // 4. Generate Rust code from the IR
    // 5. Use rustc to compile the generated Rust code to native machine code
    
    Err(CompilerError::CompilationError("Not implemented yet".to_string()))
} 