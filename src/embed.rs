use std::cell::RefCell;
use std::rc::Rc;

use crate::error::LaminaError;
use crate::evaluator;
use crate::lexer;
use crate::parser;
use crate::value::{Environment, Value};

/// A wrapper that represents a Lamina interpreter instance
pub struct Interpreter {
    env: Rc<RefCell<Environment>>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Create a new Lamina interpreter with a fresh environment
    pub fn new() -> Self {
        let env = evaluator::setup_initial_env();
        
        // Load any registered FFI functions
        if let Err(e) = crate::ffi::load_ffi_functions(&env) {
            eprintln!("Warning: Failed to load FFI functions: {}", e);
        }
        
        Interpreter { env }
    }

    /// Evaluate a string of Lamina code and return the result
    pub fn eval(&self, code: &str) -> Result<Value, LaminaError> {
        let tokens = lexer::lex(code)?;
        let expr = parser::parse(&tokens)?;
        evaluator::eval_with_env(expr, self.env.clone())
    }

    /// Define a variable in the interpreter's environment
    pub fn define(&self, name: &str, value: Value) {
        evaluator::environment::define_variable(name, value, self.env.clone());
    }

    /// Set an existing variable in the interpreter's environment
    pub fn set(&self, name: &str, value: Value) -> Result<(), LaminaError> {
        evaluator::environment::set_variable(name, value, self.env.clone())
    }

    /// Get a variable from the interpreter's environment
    pub fn get(&self, name: &str) -> Option<Value> {
        evaluator::environment::lookup_variable(name, self.env.clone())
    }

    /// Call a Lamina procedure with the given arguments
    pub fn call(&self, proc_name: &str, args: Vec<Value>) -> Result<Value, LaminaError> {
        // Look up the procedure
        let proc = self.get(proc_name).ok_or_else(|| {
            LaminaError::Runtime(format!("Procedure not found: {}", proc_name))
        })?;

        println!("Debug - Calling procedure '{}' of type: {:?}", proc_name, proc);

        // Call the procedure
        match proc {
            Value::Procedure(p) => p(args).map_err(|e| LaminaError::Runtime(e)),
            Value::RustFn(f, _) => f(args).map_err(|e| LaminaError::Runtime(e)),
            _ => Err(LaminaError::Runtime(format!(
                "{} is not a procedure: {:?}",
                proc_name, proc
            ))),
        }
    }

    /// Register a Rust function in the Lamina environment
    pub fn register_function<F>(&self, name: &str, func: F)
    where
        F: Fn(Vec<Value>) -> Result<Value, String> + 'static,
    {
        self.env.borrow_mut().bindings.insert(
            name.to_string(),
            crate::ffi::create_rust_fn(name, func),
        );
    }

    /// Get access to the interpreter's environment
    pub fn environment(&self) -> Rc<RefCell<Environment>> {
        self.env.clone()
    }
}

/// Convenience function to create and initialize a Lamina interpreter
pub fn init() -> Interpreter {
    Interpreter::new()
}

/// Convenience function to evaluate a string of Lamina code
pub fn eval(code: &str) -> Result<Value, LaminaError> {
    let interpreter = Interpreter::new();
    interpreter.eval(code)
}

/// Convenience type aliases for working with Lamina from Rust
pub mod types {
    pub use crate::value::{NumberKind, Value};
    pub use crate::ffi::{
        i64_to_value, f64_to_value, bool_to_value, string_to_value,
        value_to_i64, value_to_f64, value_to_bool, value_to_string,
    };
} 