//! Backend for code generation
//! 
//! This module defines the backend interfaces for code generation from the IR.

use lamina_ir::ir::{Program, Expr, Def, Type};
use crate::Result;

/// A trait for backend code generators
pub trait Backend {
    /// Initialize the backend
    fn init(&mut self) -> Result<()>;
    
    /// Generate code from a program
    fn gen_program(&mut self, program: &Program) -> Result<()>;
    
    /// Finalize code generation and write output
    fn finalize(&mut self, output_path: &str) -> Result<()>;
}

/// A backend that generates Rust code
pub struct RustBackend {
    /// Generated Rust code
    code: String,
}

impl RustBackend {
    /// Create a new Rust backend
    pub fn new() -> Self {
        Self {
            code: String::new(),
        }
    }
    
    /// Generate Rust code for a type
    fn gen_type(&mut self, ty: &Type) -> Result<String> {
        match ty {
            Type::Int(width) => {
                match width {
                    8 => Ok("i8".to_string()),
                    16 => Ok("i16".to_string()),
                    32 => Ok("i32".to_string()),
                    64 => Ok("i64".to_string()),
                    128 => Ok("i128".to_string()),
                    _ => Err(crate::CompilerError::CompilationError(
                        format!("Unsupported integer width: {}", width)
                    )),
                }
            },
            Type::Uint(width) => {
                match width {
                    8 => Ok("u8".to_string()),
                    16 => Ok("u16".to_string()),
                    32 => Ok("u32".to_string()),
                    64 => Ok("u64".to_string()),
                    128 => Ok("u128".to_string()),
                    _ => Err(crate::CompilerError::CompilationError(
                        format!("Unsupported unsigned integer width: {}", width)
                    )),
                }
            },
            Type::Bool => Ok("bool".to_string()),
            Type::String => Ok("String".to_string()),
            Type::Bytes(size) => Ok(format!("[u8; {}]", size)),
            Type::Function(params, ret) => {
                let param_types = params.iter()
                    .map(|p| self.gen_type(p))
                    .collect::<Result<Vec<_>>>()?
                    .join(", ");
                let ret_type = self.gen_type(ret)?;
                Ok(format!("fn({}) -> {}", param_types, ret_type))
            },
            Type::UserDefined(ident) => Ok(ident.0.clone()),
            Type::Unit => Ok("()".to_string()),
        }
    }
}

impl Backend for RustBackend {
    fn init(&mut self) -> Result<()> {
        // Add Rust boilerplate
        self.code.push_str("fn main() {\n");
        Ok(())
    }
    
    fn gen_program(&mut self, program: &Program) -> Result<()> {
        // For now, just add placeholder code
        self.code.push_str("    // Generated from Lamina IR\n");
        self.code.push_str("    println!(\"Hello from Lamina!\");\n");
        
        Ok(())
    }
    
    fn finalize(&mut self, output_path: &str) -> Result<()> {
        // Close the main function
        self.code.push_str("}\n");
        
        // Write the generated code to the output file
        std::fs::write(output_path, &self.code)?;
        
        Ok(())
    }
} 