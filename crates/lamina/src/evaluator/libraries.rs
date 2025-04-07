use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::value::{Environment, Library, NumberKind, Value};

use super::environment::create_environment;
use crate::evaluator::library_manager;

// Helper functions for EVM library
pub fn check_args_count(func_name: &str, args: &[Value], expected: usize) -> Result<(), String> {
    if args.len() != expected {
        return Err(format!(
            "{} expected {} arguments, got {}",
            func_name,
            expected,
            args.len()
        ));
    }
    Ok(())
}

pub fn number_to_i64(val: &Value) -> Result<i64, String> {
    match val {
        Value::Number(n) => match n {
            NumberKind::Integer(i) => Ok(*i),
            NumberKind::Real(r) => Ok(*r as i64),
            NumberKind::Rational(num, den) => Ok(*num / *den),
        },
        _ => Err(format!("Expected number, got {}", val)),
    }
}

// Base library registration
pub fn register_base_library(env: Rc<RefCell<Environment>>) {
    let base_env = create_environment(Some(env.clone()));

    // Add basic list operations
    base_env.borrow_mut().bindings.insert(
        "append".to_string(),
        Value::Procedure(Rc::new(|_args| {
            // Implementation of append
            Ok(Value::Nil)
        })),
    );

    // Register the library in the parent environment
    env.borrow_mut().bindings.insert(
        "base".to_string(),
        Value::Library(Rc::new(RefCell::new(Library {
            name: vec!["scheme".to_string(), "base".to_string()],
            exports: vec!["append".to_string()],
            imports: vec![],
            environment: base_env,
        }))),
    );
}

// File library registration
pub fn register_file_library(env: Rc<RefCell<Environment>>) {
    let file_env = create_environment(Some(env.clone()));

    // Add file operations
    file_env.borrow_mut().bindings.insert(
        "file-exists?".to_string(),
        Value::Procedure(Rc::new(|_args| {
            // Implementation of file-exists?
            Ok(Value::Boolean(false))
        })),
    );

    // Register the library in the parent environment
    env.borrow_mut().bindings.insert(
        "file".to_string(),
        Value::Library(Rc::new(RefCell::new(Library {
            name: vec!["scheme".to_string(), "file".to_string()],
            exports: vec!["file-exists?".to_string()],
            imports: vec![],
            environment: file_env,
        }))),
    );
}

// Math library registration
pub fn register_math_library(env: Rc<RefCell<Environment>>) {
    let math_env = create_environment(Some(env.clone()));

    // Add math operations
    math_env.borrow_mut().bindings.insert(
        "abs".to_string(),
        Value::Procedure(Rc::new(|args| {
            check_args_count("abs", &args, 1)?;
            match &args[0] {
                Value::Number(n) => match n {
                    NumberKind::Integer(i) => Ok(Value::Number(NumberKind::Integer(i.abs()))),
                    NumberKind::Real(r) => Ok(Value::Number(NumberKind::Real(r.abs()))),
                    NumberKind::Rational(num, den) => {
                        Ok(Value::Number(NumberKind::Rational(num.abs(), *den)))
                    }
                },
                _ => Err("abs: expected number".to_string()),
            }
        })),
    );

    // Register the library in the parent environment
    env.borrow_mut().bindings.insert(
        "math".to_string(),
        Value::Library(Rc::new(RefCell::new(Library {
            name: vec!["scheme".to_string(), "math".to_string()],
            exports: vec!["abs".to_string()],
            imports: vec![],
            environment: math_env,
        }))),
    );
}

// EVM library registration
pub fn register_evm_library(env: Rc<RefCell<Environment>>) {
    let evm_env = create_environment(Some(env.clone()));

    // Storage operations
    evm_env.borrow_mut().bindings.insert(
        "storage-load".to_string(),
        Value::Procedure(Rc::new(|args| {
            check_args_count("storage-load", &args, 1)?;
            let _slot = number_to_i64(&args[0])?;
            // This is a mock implementation since we're focusing on compilation
            Ok(Value::Number(NumberKind::Integer(0)))
        })),
    );

    evm_env.borrow_mut().bindings.insert(
        "storage-store".to_string(),
        Value::Procedure(Rc::new(|args| {
            check_args_count("storage-store", &args, 2)?;
            let _slot = number_to_i64(&args[0])?;
            let _value = &args[1];
            // This is a mock implementation since we're focusing on compilation
            Ok(Value::Nil)
        })),
    );

    // Contract execution control
    evm_env.borrow_mut().bindings.insert(
        "revert".to_string(),
        Value::Procedure(Rc::new(|args| {
            check_args_count("revert", &args, 1)?;
            // This is a mock implementation since we're focusing on compilation
            Ok(Value::Nil)
        })),
    );

    // Register the library in the parent environment
    env.borrow_mut().bindings.insert(
        "evm".to_string(),
        Value::Library(Rc::new(RefCell::new(Library {
            name: vec!["evm".to_string()],
            exports: vec![
                "storage-load".to_string(),
                "storage-store".to_string(),
                "revert".to_string(),
            ],
            imports: vec![],
            environment: evm_env,
        }))),
    );
}

// Setup all libraries
pub fn setup_libraries(env: Rc<RefCell<Environment>>) -> Result<(), Error> {
    register_base_library(env.clone());
    register_file_library(env.clone());
    register_math_library(env.clone());
    register_evm_library(env.clone());
    Ok(())
}

// Define-library special form implementation
pub fn eval_define_library(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, Error> {
    if let Value::Pair(name_pair) = args {
        let name_expr = &name_pair.0;
        let decls = name_pair.1.clone();

        let lib_name = extract_library_name(name_expr)?;
        if lib_name.is_empty() {
            return Err(Error::Runtime("Library name cannot be empty".into()));
        }

        // Create the library environment as a child of the parent environment
        let lib_env = create_environment(Some(env.clone()));

        // Process library declarations
        let mut exports = Vec::new();
        let mut imports = Vec::new();

        let mut remaining_decls = decls;
        while let Value::Pair(decl_pair) = remaining_decls {
            let decl = decl_pair.0.clone();
            remaining_decls = decl_pair.1.clone();

            if let Value::Pair(decl_pair) = decl {
                if let Value::Symbol(ref decl_type) = decl_pair.0 {
                    let decl_contents = decl_pair.1.clone();

                    match decl_type.as_str() {
                        "export" => {
                            let export_list = extract_exports(&decl_contents)?;
                            exports.extend(export_list);
                        }
                        "import" => {
                            let import_list = extract_imports(&decl_contents)?;
                            imports.extend(import_list);
                        }
                        "begin" => {
                            // Evaluate the body in the library's environment
                            eval_begin(decl_contents, lib_env.clone())?;
                        }
                        _ => {
                            return Err(Error::Runtime(format!(
                                "Unknown library declaration: {}",
                                decl_type
                            )));
                        }
                    }
                }
            }
        }

        // Create the library
        let library = Library {
            name: lib_name.clone(),
            exports,
            imports,
            environment: lib_env.clone(),
        };

        // Create the library value
        let lib_rc = Rc::new(RefCell::new(library));
        let lib_value = Value::Library(lib_rc.clone());

        // Register the library with the library manager
        library_manager::register_library(lib_rc);

        // Register the library in the global environment
        let mut path = String::new();
        for (i, part) in lib_name.iter().enumerate() {
            if i > 0 {
                path.push(' ');
            }
            path.push_str(part);
        }

        // Register the library in the hierarchical structure
        if lib_name.len() == 1 {
            env.borrow_mut()
                .bindings
                .insert(lib_name[0].clone(), lib_value.clone());
        } else {
            // For nested libraries, we need to find or create the parent libraries
            let mut current_env = env.clone();
            for (i, part) in lib_name.iter().enumerate() {
                if i == lib_name.len() - 1 {
                    // Last part, insert library
                    current_env
                        .borrow_mut()
                        .bindings
                        .insert(part.clone(), lib_value.clone());
                } else {
                    // Get or create parent library
                    let parent_lib = {
                        let current_env_ref = current_env.borrow();
                        match current_env_ref.bindings.get(part) {
                            Some(Value::Library(lib)) => lib.clone(),
                            _ => {
                                // We need to drop the current borrow before creating a new one
                                drop(current_env_ref);

                                // Create a new parent library
                                let parent_lib = Library {
                                    name: lib_name[0..=i].to_vec(),
                                    exports: Vec::new(),
                                    imports: Vec::new(),
                                    environment: create_environment(Some(current_env.clone())),
                                };
                                let parent_lib_value = Rc::new(RefCell::new(parent_lib));
                                current_env
                                    .borrow_mut()
                                    .bindings
                                    .insert(part.clone(), Value::Library(parent_lib_value.clone()));
                                parent_lib_value
                            }
                        }
                    };
                    current_env = parent_lib.borrow().environment.clone();
                }
            }
        }

        // Return information about the library
        let library_info = format!("#<library:{}>", lib_name.join(" "));
        Ok(Value::String(library_info))
    } else {
        Err(Error::Runtime("Malformed define-library form".into()))
    }
}

// Evaluate a begin expression (sequence of expressions) in a given environment
fn eval_begin(args: Value, env: Rc<RefCell<Environment>>) -> Result<Value, Error> {
    let mut result = Value::Nil;
    let mut remaining_args = args;

    while let Value::Pair(pair) = remaining_args {
        result = super::eval_with_env(pair.0.clone(), env.clone())?;
        remaining_args = pair.1.clone();
    }

    Ok(result)
}

// Helper function to extract library name from library form
fn extract_library_name(name_expr: &Value) -> Result<Vec<String>, Error> {
    let mut result = Vec::new();
    let mut name = name_expr.clone();

    while let Value::Pair(name_pair) = name {
        if let Value::Symbol(s) = &name_pair.0 {
            result.push(s.clone());
        } else {
            return Err(Error::Runtime(
                "Library name must be a list of symbols".into(),
            ));
        }
        name = name_pair.1.clone();
    }

    Ok(result)
}

// Helper function to extract exports from export form
fn extract_exports(export_expr: &Value) -> Result<Vec<String>, Error> {
    let mut result = Vec::new();
    let mut exports = export_expr.clone();

    while let Value::Pair(export_pair) = exports {
        if let Value::Symbol(s) = &export_pair.0 {
            result.push(s.clone());
        } else {
            return Err(Error::Runtime("Exports must be symbols".into()));
        }
        exports = export_pair.1.clone();
    }

    Ok(result)
}

// Helper function to extract imports from import form
fn extract_imports(import_expr: &Value) -> Result<Vec<Vec<String>>, Error> {
    let mut result = Vec::new();
    let mut imports = import_expr.clone();

    while let Value::Pair(import_pair) = imports {
        let lib_name = extract_library_name(&import_pair.0)?;
        result.push(lib_name);
        imports = import_pair.1.clone();
    }

    Ok(result)
}
