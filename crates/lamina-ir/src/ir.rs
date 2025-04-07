//! Intermediate Representation core types

use std::collections::HashMap;

/// A unique identifier for a variable or definition
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub String);

/// Type representation in the IR
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Integer types with bit width
    Int(usize),
    /// Unsigned integer types with bit width
    Uint(usize),
    /// Boolean type
    Bool,
    /// String type (for compile-time constants)
    String,
    /// Byte array with fixed size
    Bytes(usize),
    /// Function type with parameter types and return type
    Function(Vec<Type>, Box<Type>),
    /// User-defined type
    UserDefined(Ident),
    /// Unit type (used for expressions with no return value)
    Unit,
}

/// Expression nodes in the IR
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal
    IntLit(i64),
    /// Unsigned integer literal
    UintLit(u64),
    /// Boolean literal
    BoolLit(bool),
    /// String literal
    StringLit(String),
    /// Bytes literal
    BytesLit(Vec<u8>),
    /// Variable reference
    Var(Ident),
    /// Function call
    Call(Box<Expr>, Vec<Expr>),
    /// Lambda expression
    Lambda(Vec<(Ident, Type)>, Box<Expr>),
    /// If expression
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    /// Let binding
    Let(Ident, Box<Expr>, Box<Expr>),
    /// Binary operation
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    /// Unary operation
    UnOp(UnOp, Box<Expr>),
}

/// Binary operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
}

/// Unary operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Not,
    Neg,
}

/// Top-level definition
#[derive(Debug, Clone, PartialEq)]
pub enum Def {
    /// Function definition
    Function {
        name: Ident,
        params: Vec<(Ident, Type)>,
        return_type: Type,
        body: Expr,
    },
    /// Constant definition
    Const {
        name: Ident,
        ty: Type,
        value: Expr,
    },
    /// Type definition
    TypeDef {
        name: Ident,
        fields: Vec<(Ident, Type)>,
    },
}

/// A complete program in the IR
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Top-level definitions
    pub defs: Vec<Def>,
    /// Module metadata
    pub metadata: HashMap<String, String>,
}

impl Program {
    /// Create a new empty program
    pub fn new() -> Self {
        Self {
            defs: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a definition to the program
    pub fn add_def(&mut self, def: Def) {
        self.defs.push(def);
    }
    
    /// Add metadata to the program
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
} 