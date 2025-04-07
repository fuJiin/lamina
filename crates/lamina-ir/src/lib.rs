//! Lamina Intermediate Representation
//! 
//! This crate defines the intermediate representation (IR) for the Lamina language.
//! The IR serves as a bridge between the frontend parser and the various backend
//! compilers (native machine code, EVM, etc.).

use thiserror::Error;

pub mod ir;
pub mod visitor;
pub mod transforms;

#[derive(Debug, Error)]
pub enum IrError {
    #[error("IR conversion error: {0}")]
    ConversionError(String),
    
    #[error("Invalid IR: {0}")]
    InvalidIr(String),
}

/// Result type for IR operations
pub type Result<T> = std::result::Result<T, IrError>;

/// Re-export the main IR types
pub use ir::{Expr, Def, Program, Type}; 