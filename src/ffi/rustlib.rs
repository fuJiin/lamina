use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{Environment, Value};

/// A module or set of Rust functions that can be imported into Lamina
pub struct RustModule {
    name: String,
    functions: HashMap<String, Rc<dyn Fn(Vec<Value>) -> Result<Value, String>>>,
}

impl RustModule {
    pub fn new(name: &str) -> Self {
        RustModule {
            name: name.to_string(),
            functions: HashMap::new(),
        }
    }

    /// Add a function to the module
    pub fn add_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(Vec<Value>) -> Result<Value, String> + 'static,
    {
        self.functions.insert(name.to_string(), Rc::new(func));
    }

    /// Import all functions from this module into the given environment
    pub fn import_into_env(&self, env: &Rc<RefCell<Environment>>) {
        for (name, func) in &self.functions {
            // Create qualified name: module-name/function-name
            let qualified_name = format!("{}/{}", self.name, name);
            
            env.borrow_mut().bindings.insert(
                qualified_name,
                Value::RustFn(func.clone(), format!("{}.{}", self.name, name)),
            );
        }
    }
}

// Registry of all available Rust modules
thread_local! {
    static MODULES: RefCell<HashMap<String, RustModule>> = RefCell::new(HashMap::new());
}

/// Register a Rust module for use in Lamina
pub fn register_module(module: RustModule) {
    MODULES.with(|modules| {
        modules.borrow_mut().insert(module.name.clone(), module);
    });
}

/// Import a Rust module into the given environment
pub fn import_module(module_name: &str, env: &Rc<RefCell<Environment>>) -> Result<(), String> {
    MODULES.with(|modules| {
        let modules = modules.borrow();
        if let Some(module) = modules.get(module_name) {
            module.import_into_env(env);
            Ok(())
        } else {
            Err(format!("Rust module not found: {}", module_name))
        }
    })
}

/// Utility function to create and register a module in one step
pub fn create_module<F>(name: &str, setup_fn: F)
where
    F: FnOnce(&mut RustModule),
{
    let mut module = RustModule::new(name);
    setup_fn(&mut module);
    register_module(module);
} 