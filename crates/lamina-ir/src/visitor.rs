//! Visitor pattern for the IR

use crate::ir::{Program, Def, Expr, Type, BinOp, UnOp, Ident};
use crate::Result;

/// A visitor for the IR
pub trait Visitor<T> {
    /// Visit a program
    fn visit_program(&mut self, program: &Program) -> Result<T>;
    
    /// Visit a definition
    fn visit_def(&mut self, def: &Def) -> Result<T>;
    
    /// Visit an expression
    fn visit_expr(&mut self, expr: &Expr) -> Result<T>;
    
    /// Visit a type
    fn visit_type(&mut self, ty: &Type) -> Result<T>;
    
    /// Visit a binary operation
    fn visit_bin_op(&mut self, op: &BinOp) -> Result<T>;
    
    /// Visit a unary operation
    fn visit_un_op(&mut self, op: &UnOp) -> Result<T>;
    
    /// Visit an identifier
    fn visit_ident(&mut self, ident: &Ident) -> Result<T>;
}

/// A transformer for the IR
pub trait Transformer {
    /// Transform a program
    fn transform_program(&mut self, program: Program) -> Result<Program>;
    
    /// Transform a definition
    fn transform_def(&mut self, def: Def) -> Result<Def>;
    
    /// Transform an expression
    fn transform_expr(&mut self, expr: Expr) -> Result<Expr>;
    
    /// Transform a type
    fn transform_type(&mut self, ty: Type) -> Result<Type>;
}

/// Default implementation for the transformer
impl Transformer for () {
    fn transform_program(&mut self, program: Program) -> Result<Program> {
        Ok(program)
    }
    
    fn transform_def(&mut self, def: Def) -> Result<Def> {
        Ok(def)
    }
    
    fn transform_expr(&mut self, expr: Expr) -> Result<Expr> {
        Ok(expr)
    }
    
    fn transform_type(&mut self, ty: Type) -> Result<Type> {
        Ok(ty)
    }
} 