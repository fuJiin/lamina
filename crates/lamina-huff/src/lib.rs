//! Huff backend for the Lamina language
//!
//! This crate provides a backend for compiling Lamina IR to Huff code for the EVM.

use lamina_ir::ir::Program;
use thiserror::Error;

// The existing huff module contains the original implementation
pub mod huff;

// New modules for the IR-based backend
pub mod backend;
pub mod optimizer;

#[derive(Debug, Error)]
pub enum HuffError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("IR error: {0}")]
    IrError(#[from] lamina_ir::IrError),

    #[error("Huff generation error: {0}")]
    GenerationError(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Legacy error: {0}")]
    LegacyError(#[from] lamina::error::Error),
}

/// Result type for Huff operations
pub type Result<T> = std::result::Result<T, HuffError>;

/// Struct representing Huff compilation options
#[derive(Debug, Clone)]
pub struct HuffOptions {
    /// Path to the output directory
    pub output_dir: String,

    /// Base name for output files
    pub base_name: String,

    /// Whether to optimize the generated Huff code
    pub optimize: bool,
}

/// Compile Lamina IR to Huff code
pub fn compile_to_huff(ir: &Program, options: &HuffOptions) -> Result<String> {
    // Use the HuffBackend to generate Huff code
    let mut backend = backend::HuffBackend::new();
    let huff_code = backend.generate(ir)?;

    // If optimization is requested, apply optimizations
    if options.optimize {
        // In a real implementation, we would apply optimizations here
        // For now, just return the unoptimized code
    }

    Ok(huff_code)
}

/// Compile and save Huff code to a file
pub fn compile_and_save(ir: &Program, options: &HuffOptions) -> Result<()> {
    let huff_code = compile_to_huff(ir, options)?;

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&options.output_dir)?;

    // Write Huff code to file
    let output_path = format!("{}/{}.huff", options.output_dir, options.base_name);
    std::fs::write(&output_path, &huff_code)?;

    println!("Huff code written to {}", output_path);

    Ok(())
}

/// Adapter to use the new IR with the existing huff compiler
///
/// This is a temporary bridge until the new backend is fully implemented
pub fn compile_ir_with_legacy(_ir: &Program, _contract_name: &str) -> Result<String> {
    // This would convert the IR to a lamina::value::Value and use the existing compiler
    // For now, this is just a placeholder
    Err(HuffError::UnsupportedFeature(
        "IR to legacy conversion not yet implemented".to_string(),
    ))
}

// Re-export core lamina types used in this crate
pub use lamina;
