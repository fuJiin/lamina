pub mod bytecode;
pub mod compiler;
pub mod opcodes;
pub mod types;

use crate::error::Error;
use crate::value::Value;

/// Compiles a Lamina expression to Huff code.
///
/// # Arguments
///
/// * `expr` - The Lamina expression to compile
/// * `contract_name` - The name of the contract to generate
///
/// # Returns
///
/// A string containing the generated Huff code
pub fn compile(expr: &Value, contract_name: &str) -> Result<String, Error> {
    compiler::compile(expr, contract_name)
}

/// Compiles and outputs Huff code to a file.
///
/// # Arguments
///
/// * `expr` - The Lamina expression to compile
/// * `contract_name` - The name of the contract to generate
/// * `output_path` - Path where the Huff file should be saved
///
/// # Returns
///
/// Success or error
pub fn compile_to_file(expr: &Value, contract_name: &str, output_path: &str) -> Result<(), Error> {
    let huff_code = compile(expr, contract_name)?;
    std::fs::write(output_path, huff_code).map_err(|e| Error::IO(e.to_string()))?;
    Ok(())
}
