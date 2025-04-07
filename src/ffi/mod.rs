pub mod rustlib;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::Error;
use crate::value::{Environment, Value};

/// Type alias for Rust functions callable from Lamina
pub type RustFunction = Rc<dyn Fn(Vec<Value>) -> Result<Value, String>>;

/// A registry to hold Rust foreign functions that can be called from Lamina
pub struct FFIRegistry {
    functions: HashMap<String, RustFunction>,
}

impl Default for FFIRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FFIRegistry {
    pub fn new() -> Self {
        FFIRegistry {
            functions: HashMap::new(),
        }
    }

    /// Register a Rust function that can be called from Lamina
    pub fn register<F>(&mut self, name: &str, func: F)
    where
        F: Fn(Vec<Value>) -> Result<Value, String> + 'static,
    {
        self.functions.insert(name.to_string(), Rc::new(func));
    }

    /// Get a function by name
    pub fn get(&self, name: &str) -> Option<RustFunction> {
        self.functions.get(name).cloned()
    }

    /// Load all registered functions into a Lamina environment
    pub fn load_into_env(&self, env: &Rc<RefCell<Environment>>) -> Result<(), Error> {
        for (name, func) in &self.functions {
            env.borrow_mut()
                .bindings
                .insert(name.clone(), create_rust_fn_from_rc(name, func.clone()));
        }
        Ok(())
    }
}

// Global registry
thread_local! {
    static FFI_REGISTRY: RefCell<FFIRegistry> = RefCell::new(FFIRegistry::new());
}

/// Register a Rust function that can be called from Lamina
pub fn register_function<F>(name: &str, func: F)
where
    F: Fn(Vec<Value>) -> Result<Value, String> + 'static,
{
    FFI_REGISTRY.with(|registry| {
        registry.borrow_mut().register(name, func);
    });
}

/// Load all registered functions into the given environment
pub fn load_ffi_functions(env: &Rc<RefCell<Environment>>) -> Result<(), Error> {
    FFI_REGISTRY.with(|registry| registry.borrow().load_into_env(env))
}

// Convenience functions for converting between Rust and Lamina types

/// Convert a Rust i64 to a Lamina Value
pub fn i64_to_value(value: i64) -> Value {
    Value::Number(crate::value::NumberKind::Integer(value))
}

/// Convert a Rust f64 to a Lamina Value
pub fn f64_to_value(value: f64) -> Value {
    Value::Number(crate::value::NumberKind::Real(value))
}

/// Convert a Rust bool to a Lamina Value
pub fn bool_to_value(value: bool) -> Value {
    Value::Boolean(value)
}

/// Convert a Rust String to a Lamina Value
pub fn string_to_value(value: String) -> Value {
    Value::String(value)
}

/// Try to convert a Lamina Value to a Rust i64
pub fn value_to_i64(value: &Value) -> Result<i64, String> {
    match value {
        Value::Number(crate::value::NumberKind::Integer(i)) => Ok(*i),
        Value::Number(crate::value::NumberKind::Real(r)) => {
            if r.fract() == 0.0 {
                Ok(*r as i64)
            } else {
                Err(format!("Cannot convert {} to integer", r))
            }
        }
        _ => Err(format!("Cannot convert {:?} to integer", value)),
    }
}

/// Try to convert a Lamina Value to a Rust f64
pub fn value_to_f64(value: &Value) -> Result<f64, String> {
    match value {
        Value::Number(n) => Ok(n.as_f64()),
        _ => Err(format!("Cannot convert {:?} to float", value)),
    }
}

/// Try to convert a Lamina Value to a Rust bool
pub fn value_to_bool(value: &Value) -> Result<bool, String> {
    match value {
        Value::Boolean(b) => Ok(*b),
        _ => Err(format!("Cannot convert {:?} to boolean", value)),
    }
}

/// Try to convert a Lamina Value to a Rust String
pub fn value_to_string(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Symbol(s) => Ok(s.clone()),
        _ => Err(format!("Cannot convert {:?} to string", value)),
    }
}

/// Convenience function to create a RustFn value directly from a function
/// This helps prevent "dead code" warnings since we're explicitly constructing RustFn variants
#[allow(dead_code)]
pub fn create_rust_fn<F>(name: &str, func: F) -> Value
where
    F: Fn(Vec<Value>) -> Result<Value, String> + 'static,
{
    Value::RustFn(Rc::new(func), name.to_string())
}

/// Convenience function to create a RustFn value from an already Rc-wrapped function
#[allow(dead_code)]
pub fn create_rust_fn_from_rc(
    name: &str,
    func: Rc<dyn Fn(Vec<Value>) -> Result<Value, String>>,
) -> Value {
    Value::RustFn(func, name.to_string())
}
