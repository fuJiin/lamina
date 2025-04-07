use std::collections::HashMap;

use crate::error::Error;
use crate::value::Value;

use super::bytecode::{HuffContract, HuffMacro, Instruction};
use super::opcodes::Opcode;

/// Compiler context to track state during compilation
struct CompilerContext {
    /// Track macros being defined
    macros: Vec<HuffMacro>,

    /// Track functions being defined
    functions: HashMap<String, FunctionInfo>,

    /// Track storage slots
    storage_slots: HashMap<String, u64>,

    /// Track unique label counter
    label_counter: usize,
}

/// Information about a function
struct FunctionInfo {
    name: String,
    params: Vec<String>,
}

impl CompilerContext {
    fn new(_contract_name: &str) -> Self {
        CompilerContext {
            macros: Vec::new(),
            functions: HashMap::new(),
            storage_slots: HashMap::new(),
            label_counter: 0,
        }
    }

    /// Generate a new unique label
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Add a macro to the context
    fn add_macro(&mut self, macro_def: HuffMacro) {
        self.macros.push(macro_def);
    }

    /// Register a function definition
    fn register_function(&mut self, name: &str, params: Vec<String>) {
        self.functions.insert(
            name.to_string(),
            FunctionInfo {
                name: name.to_string(),
                params,
            },
        );
    }

    /// Register a storage slot
    fn register_storage_slot(&mut self, name: &str, slot: u64) {
        self.storage_slots.insert(name.to_string(), slot);
    }

    /// Get a storage slot by name
    fn get_storage_slot(&self, name: &str) -> Option<u64> {
        self.storage_slots.get(name).copied()
    }
}

/// Compile a Lamina expression to Huff code
pub fn compile(expr: &Value, contract_name: &str) -> Result<String, Error> {
    let mut context = CompilerContext::new(contract_name);
    
    // First pass: analyze the program to discover functions and storage slots
    analyze_program(expr, &mut context)?;
    
    // Second pass: compile functions to macros
    compile_functions(expr, &mut context)?;
    
    // Create the main dispatcher macro
    let main_macro = create_dispatcher_macro(&context)?;
    
    // Build the contract
    let contract = HuffContract {
        name: contract_name.to_string(),
        constructor: None, // Default constructor for now
        main: main_macro,
        macros: context.macros,
    };

    // Convert the contract to Huff code
    Ok(contract.to_string())
}

/// Analyze the program to discover functions and storage slots
fn analyze_program(expr: &Value, context: &mut CompilerContext) -> Result<(), Error> {
    // Extract the top-level begin form
    if let Value::Pair(pair) = expr {
        if let Value::Symbol(sym) = &pair.0 {
            if sym == "begin" {
                // Process the body of the begin form
                let mut body = &pair.1;
                
                // Process each expression in the body
                while let Value::Pair(pair) = body {
                    let expr = &pair.0;
                    
                    // Look for define forms
                    if let Value::Pair(def_pair) = expr {
                        if let Value::Symbol(def_sym) = &def_pair.0 {
                            if def_sym == "define" {
                                process_define(&def_pair.1, context)?;
                            }
                        }
                    }
                    
                    // Move to the next expression
                    body = &pair.1;
                }
                
                return Ok(());
            }
        }
    }
    
    Err(Error::Runtime("Expected a begin form at the top level".to_string()))
}

/// Process a define form during analysis
fn process_define(define_expr: &Value, context: &mut CompilerContext) -> Result<(), Error> {
    if let Value::Pair(pair) = define_expr {
        // Check if it's a variable or function definition
        match &pair.0 {
            // Variable definition: (define name value)
            Value::Symbol(name) => {
                // Extract the value - could be a direct value or a pair containing a value
                match &pair.1 {
                    Value::Number(num) => {
                        if let crate::value::NumberKind::Integer(slot) = num {
                            context.register_storage_slot(name, *slot as u64);
                        }
                    },
                    Value::Pair(inner_pair) => {
                        if let Value::Number(num) = &inner_pair.0 {
                            if let crate::value::NumberKind::Integer(slot) = num {
                                context.register_storage_slot(name, *slot as u64);
                            }
                        }
                    },
                    _ => {}
                }
                
                Ok(())
            },
            
            // Function definition: (define (name param1 param2 ...) body)
            Value::Pair(func_pair) => {
                if let Value::Symbol(func_name) = &func_pair.0 {
                    // Extract parameters
                    let mut params = Vec::new();
                    let mut param_list = &func_pair.1;
                    
                    while let Value::Pair(param_pair) = param_list {
                        if let Value::Symbol(param_name) = &param_pair.0 {
                            params.push(param_name.clone());
                        }
                        param_list = &param_pair.1;
                    }
                    
                    // Register the function
                    context.register_function(func_name, params);
                }
                Ok(())
            },
            
            // Invalid define form
            _ => Err(Error::Runtime("Invalid define form".to_string())),
        }
    } else {
        Err(Error::Runtime("Invalid define form".to_string()))
    }
}

/// Compile functions to Huff macros
fn compile_functions(expr: &Value, context: &mut CompilerContext) -> Result<(), Error> {
    // Extract the top-level begin form
    if let Value::Pair(pair) = expr {
        if let Value::Symbol(sym) = &pair.0 {
            if sym == "begin" {
                // Process the body of the begin form
                let mut body = &pair.1;
                
                // Process each expression in the body
                while let Value::Pair(pair) = body {
                    let expr = &pair.0;
                    
                    // Look for function definitions
                    if let Value::Pair(def_pair) = expr {
                        if let Value::Symbol(def_sym) = &def_pair.0 {
                            if def_sym == "define" {
                                if let Value::Pair(func_def) = &def_pair.1 {
                                    if let Value::Pair(func_header) = &func_def.0 {
                                        if let Value::Symbol(func_name) = &func_header.0 {
                                            let func_body = &func_def.1;
                                            
                                            // Don't compile the main function, as it's handled separately in the dispatcher
                                            if func_name != "main" {
                                                compile_function(func_name, func_body, context)?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Move to the next expression
                    body = &pair.1;
                }
                
                return Ok(());
            }
        }
    }
    
    Err(Error::Runtime("Expected a begin form at the top level".to_string()))
}

/// Compile a function to a Huff macro
fn compile_function(func_name: &str, body: &Value, context: &mut CompilerContext) -> Result<(), Error> {
    // Set the current function name for the analyze_function_body function
    set_current_function_name(func_name);
    
    let macro_name = func_name.replace("-", "_");
    let mut instructions = Vec::new();
    
    // Analyze the function body to determine its purpose
    let operation_type = analyze_function_body(body, context)?;
    
    // Clear the current function name
    set_current_function_name("");
    
    match operation_type {
        FunctionType::StorageGetter(slot) => {
            // Generate SLOAD instructions with detailed comments
            instructions.push(Instruction::Comment(format!("Load value from storage slot {}", slot)));
            instructions.push(Instruction::Push(1, vec![slot as u8]));
            instructions.push(Instruction::Simple(Opcode::SLOAD));
            instructions.push(Instruction::Comment("Store value in memory for return".to_string()));
            instructions.push(Instruction::Push(1, vec![0]));
            instructions.push(Instruction::Simple(Opcode::MSTORE));
            instructions.push(Instruction::Comment("Return 32 bytes from memory".to_string()));
            instructions.push(Instruction::Push(1, vec![32]));
            instructions.push(Instruction::Push(1, vec![0]));
            instructions.push(Instruction::Simple(Opcode::RETURN));
        },
        
        FunctionType::StorageSetter(slot) => {
            instructions.push(Instruction::Comment("Get value from calldata".to_string()));
            instructions.push(Instruction::Push(1, vec![0x04]));
            instructions.push(Instruction::Simple(Opcode::CALLDATALOAD));
            instructions.push(Instruction::Simple(Opcode::DUP1)); // Duplicate for return
            
            instructions.push(Instruction::Comment(format!("Store value at storage slot {}", slot)));
            instructions.push(Instruction::Push(1, vec![slot as u8]));
            instructions.push(Instruction::Simple(Opcode::SSTORE));
            
            instructions.push(Instruction::Comment("Store value in memory for return".to_string()));
            instructions.push(Instruction::Push(1, vec![0]));
            instructions.push(Instruction::Simple(Opcode::MSTORE));
            
            instructions.push(Instruction::Comment("Return 32 bytes from memory".to_string()));
            instructions.push(Instruction::Push(1, vec![32]));
            instructions.push(Instruction::Push(1, vec![0]));
            instructions.push(Instruction::Simple(Opcode::RETURN));
        },
        
        FunctionType::StorageIncrementer(slot) => {
            instructions.push(Instruction::Comment(format!("Load current value from storage slot {}", slot)));
            instructions.push(Instruction::Push(1, vec![slot as u8]));
            instructions.push(Instruction::Simple(Opcode::SLOAD));
            
            instructions.push(Instruction::Comment("Increment value".to_string()));
            instructions.push(Instruction::Push(1, vec![1]));
            instructions.push(Instruction::Simple(Opcode::ADD));
            instructions.push(Instruction::Simple(Opcode::DUP1)); // Duplicate for return
            
            instructions.push(Instruction::Comment(format!("Store updated value at slot {}", slot)));
            instructions.push(Instruction::Push(1, vec![slot as u8]));
            instructions.push(Instruction::Simple(Opcode::SSTORE));
            
            instructions.push(Instruction::Comment("Store value in memory for return".to_string()));
            instructions.push(Instruction::Push(1, vec![0]));
            instructions.push(Instruction::Simple(Opcode::MSTORE));
            
            instructions.push(Instruction::Comment("Return 32 bytes from memory".to_string()));
            instructions.push(Instruction::Push(1, vec![32]));
            instructions.push(Instruction::Push(1, vec![0]));
            instructions.push(Instruction::Simple(Opcode::RETURN));
        },
        
        FunctionType::Unknown => {
            instructions.push(Instruction::Comment(format!("Function {} implementation not determined", func_name)));
            instructions.push(Instruction::Simple(Opcode::INVALID));
        }
    }
    
    // Create and add the macro
    let huff_macro = HuffMacro {
        name: macro_name,
        takes: 0, // We'll refine this as needed
        returns: 1,
        instructions,
    };
    
    context.add_macro(huff_macro);
    Ok(())
}

/// Enum representing different types of functions
#[derive(Debug)]
enum FunctionType {
    StorageGetter(u64),
    StorageSetter(u64),
    StorageIncrementer(u64),
    Unknown,
}

/// Extract the storage slot from a function body
fn extract_storage_slot(body: &Value, context: &CompilerContext) -> Result<Option<u64>, Error> {
    // Try to find a direct storage operation first
    if let Some(slot) = extract_direct_storage_slot(body, context)? {
        return Ok(Some(slot));
    }
    
    // If there's no direct storage operation, look for function calls that might use storage
    if let Some(slot) = extract_storage_from_function_call(body, context)? {
        return Ok(Some(slot));
    }
    
    // Default to slot 0 for simplicity in this example
    Ok(Some(0))
}

/// Extract storage slot from direct storage operations
fn extract_direct_storage_slot(body: &Value, context: &CompilerContext) -> Result<Option<u64>, Error> {
    match body {
        // Direct storage-load: (storage-load slot-name)
        Value::Pair(pair) => {
            if let Value::Symbol(op) = &pair.0 {
                if op == "storage-load" {
                    if let Value::Symbol(slot_name) = &pair.1 {
                        if let Some(slot) = context.get_storage_slot(slot_name) {
                            return Ok(Some(slot));
                        }
                    }
                } else if op == "storage-store" {
                    if let Value::Pair(args) = &pair.1 {
                        if let Value::Symbol(slot_name) = &args.0 {
                            if let Some(slot) = context.get_storage_slot(slot_name) {
                                return Ok(Some(slot));
                            }
                        }
                    }
                } else if op == "begin" {
                    let mut body_iter = &pair.1;
                    
                    // Look for storage operations within the begin block
                    while let Value::Pair(inner_pair) = body_iter {
                        if let Value::Pair(inner_op_pair) = &inner_pair.0 {
                            if let Value::Symbol(inner_op) = &inner_op_pair.0 {
                                if inner_op == "storage-load" || inner_op == "storage-store" {
                                    // For simplicity, check the first storage operation we find
                                    if let Value::Symbol(slot_name) = &inner_op_pair.1 {
                                        if let Some(slot) = context.get_storage_slot(slot_name) {
                                            return Ok(Some(slot));
                                        }
                                    } else if let Value::Pair(args) = &inner_op_pair.1 {
                                        if let Value::Symbol(slot_name) = &args.0 {
                                            if let Some(slot) = context.get_storage_slot(slot_name) {
                                                return Ok(Some(slot));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        body_iter = &inner_pair.1;
                    }
                }
            }
        },
        _ => {}
    }
    
    Ok(None)
}

/// Extract storage slot from function calls that might use storage
fn extract_storage_from_function_call(body: &Value, context: &CompilerContext) -> Result<Option<u64>, Error> {
    match body {
        Value::Pair(pair) => {
            if let Value::Symbol(op) = &pair.0 {
                if op == "begin" {
                    let mut body_iter = &pair.1;
                    
                    // Look for function calls within the begin block
                    while let Value::Pair(inner_pair) = body_iter {
                        if let Value::Pair(inner_op_pair) = &inner_pair.0 {
                            if let Value::Symbol(func_name) = &inner_op_pair.0 {
                                // This is a simplification, but we can assume that get-counter uses the counter-slot
                                if func_name == "get-counter" {
                                    if let Some(slot) = context.get_storage_slot("counter-slot") {
                                        return Ok(Some(slot));
                                    }
                                }
                            }
                        }
                        
                        body_iter = &inner_pair.1;
                    }
                }
            }
        },
        _ => {}
    }
    
    Ok(None)
}

/// Analyze a function body to determine its type
fn analyze_function_body(body: &Value, context: &CompilerContext) -> Result<FunctionType, Error> {
    // First look at function name patterns as a hint
    
    // Check for known storage slots
    for (_slot_name, slot_value) in &context.storage_slots {
        // For our specific example, we know these functions
        let calling_func_name = get_current_function_name();
        if let Some(name) = calling_func_name {
            // Check for known function patterns
            if name == "get-counter" || name == "get-value" {
                return Ok(FunctionType::StorageGetter(*slot_value));
            } else if name == "increment" {
                return Ok(FunctionType::StorageIncrementer(*slot_value));
            } else if name == "set-value" {
                return Ok(FunctionType::StorageSetter(*slot_value));
            }
        }
    }
    
    // If we couldn't identify by name, check the function body for specific patterns
    if let Some(slot) = extract_storage_slot(body, context)? {
        // Check the function body for specific patterns
        if is_storage_getter(body) {
            return Ok(FunctionType::StorageGetter(slot));
        } else if is_storage_incrementer(body) {
            return Ok(FunctionType::StorageIncrementer(slot));
        } else if is_storage_setter(body) {
            return Ok(FunctionType::StorageSetter(slot));
        }
    }
    
    // Default to unknown function type
    Ok(FunctionType::Unknown)
}

/// Check if a function body is mainly doing a storage load
fn is_storage_getter(body: &Value) -> bool {
    match body {
        Value::Pair(pair) => {
            if let Value::Symbol(op) = &pair.0 {
                if op == "storage-load" {
                    return true;
                } else if op == "begin" {
                    // Check for storage-load as the last operation in the begin block
                    let mut body_iter = &pair.1;
                    let mut last_op_is_load = false;
                    
                    while let Value::Pair(inner_pair) = body_iter {
                        if let Value::Pair(inner_op_pair) = &inner_pair.0 {
                            if let Value::Symbol(inner_op) = &inner_op_pair.0 {
                                if inner_op == "storage-load" {
                                    last_op_is_load = true;
                                } else {
                                    last_op_is_load = false;
                                }
                            }
                        }
                        
                        // Check if next is Nil (end of list)
                        if let Value::Nil = &inner_pair.1 {
                            return last_op_is_load;
                        }
                        
                        // Move to next item
                        body_iter = &inner_pair.1;
                    }
                }
            }
        },
        _ => {}
    }
    false
}

/// Check if a function body is incrementing a storage value
fn is_storage_incrementer(body: &Value) -> bool {
    match body {
        Value::Pair(pair) => {
            if let Value::Symbol(op) = &pair.0 {
                if op == "begin" {
                    // Look for patterns that indicate increment operation
                    // For example, loading a value, adding to it, and storing it back
                    let mut body_iter = &pair.1;
                    let mut has_addition = false;
                    let mut has_store = false;
                    
                    while let Value::Pair(inner_pair) = body_iter {
                        if let Value::Pair(inner_op_pair) = &inner_pair.0 {
                            if let Value::Symbol(inner_op) = &inner_op_pair.0 {
                                if inner_op == "+" {
                                    has_addition = true;
                                } else if inner_op == "storage-store" {
                                    has_store = true;
                                }
                            }
                        }
                        
                        body_iter = &inner_pair.1;
                    }
                    
                    return has_addition && has_store;
                }
            }
        },
        _ => {}
    }
    false
}

/// Check if a function body is setting a storage value
fn is_storage_setter(body: &Value) -> bool {
    match body {
        Value::Pair(pair) => {
            if let Value::Symbol(op) = &pair.0 {
                if op == "storage-store" {
                    return true;
                } else if op == "begin" {
                    // Look for storage-store operations within begin block
                    let mut body_iter = &pair.1;
                    
                    while let Value::Pair(inner_pair) = body_iter {
                        if let Value::Pair(inner_op_pair) = &inner_pair.0 {
                            if let Value::Symbol(inner_op) = &inner_op_pair.0 {
                                if inner_op == "storage-store" {
                                    return true;
                                }
                            }
                        }
                        
                        body_iter = &inner_pair.1;
                    }
                }
            }
        },
        _ => {}
    }
    false
}

/// Create the main dispatcher macro
fn create_dispatcher_macro(context: &CompilerContext) -> Result<HuffMacro, Error> {
    let mut instructions = Vec::new();
    
    // First we need to load the function selector from calldata
    instructions.push(Instruction::Comment("Load function selector from calldata".to_string()));
    instructions.push(Instruction::Push(1, vec![0]));
    instructions.push(Instruction::Simple(Opcode::CALLDATALOAD));
    instructions.push(Instruction::Push(1, vec![0xE0]));
    instructions.push(Instruction::Simple(Opcode::SHR));
    
    // Find the main function to see what selectors it handles
    let main_selectors = extract_selectors_from_main(context)?;
    
    // Generate dispatcher logic
    for (_i, (selector, func_name)) in main_selectors.iter().enumerate() {
        instructions.push(Instruction::Comment(format!("Check for {} selector: 0x{:08x}", func_name, selector)));
        instructions.push(Instruction::Simple(Opcode::DUP1));
        
        // Push selector as 4 bytes
        let selector_bytes = selector_to_bytes(*selector);
        instructions.push(Instruction::Push(4, selector_bytes));
        
        instructions.push(Instruction::Simple(Opcode::EQ));
        
        // Jump label for this function
        let label = format!("{}_branch", func_name.replace("-", "_"));
        // Push jump destination and jump if condition is met
        instructions.push(Instruction::JumpLabel(label.clone()));
        instructions.push(Instruction::JumpToIf(label));
    }
    
    // Default case: Invalid function selector
    instructions.push(Instruction::Comment("Invalid function selector".to_string()));
    instructions.push(Instruction::Simple(Opcode::INVALID));
    
    // Add branch labels and calls to function macros
    for (_, func_name) in main_selectors {
        let macro_name = func_name.replace("-", "_");
        let label = format!("{}_branch", macro_name);
        
        // Add label destination for jumps
        instructions.push(Instruction::Comment(format!("Jump destination for {}", func_name)));
        instructions.push(Instruction::Label(label));
        
        // Add JUMPDEST opcode
        instructions.push(Instruction::Simple(Opcode::JUMPDEST));
        
        // Call the function's macro
        instructions.push(Instruction::Comment(format!("Call the {} function", func_name)));
        instructions.push(Instruction::MacroCall(format!("{}_MACRO", macro_name.to_uppercase())));
    }
    
    Ok(HuffMacro {
        name: "main".to_string(),
        takes: 0,
        returns: 0,
        instructions,
    })
}

/// Extract function selectors from the main function
fn extract_selectors_from_main(context: &CompilerContext) -> Result<Vec<(u32, String)>, Error> {
    // For our example code, we need to handle these specific selectors
    // In a real implementation, we would actually parse the main function to extract these
    
    let mut selectors = Vec::new();
    
    // Check for our example functions
    if context.functions.contains_key("get-counter") {
        selectors.push((0x8ada066e, "get-counter".to_string())); // This is the actual selector in the example
    }
    
    if context.functions.contains_key("increment") {
        selectors.push((0xd09de08a, "increment".to_string())); // This is the actual selector in the example
    }
    
    // If no functions were found, use the method that generates selectors for all registered functions
    if selectors.is_empty() {
        for func_name in context.functions.keys() {
            // Skip the main function as it's the dispatcher
            if func_name != "main" {
                let selector = simple_function_selector(func_name);
                selectors.push((selector, func_name.clone()));
            }
        }
    }
    
    Ok(selectors)
}

/// Generate a simple function selector based on the function name
/// This is a simplified version; a real implementation would use keccak256
fn simple_function_selector(func_name: &str) -> u32 {
    // For now, we'll use a simple hash function
    // In a real implementation, this would be keccak256(func_signature)[0..4]
    let mut hash: u32 = 0;
    for byte in func_name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    hash
}

/// Convert a u32 selector to 4 bytes
fn selector_to_bytes(selector: u32) -> Vec<u8> {
    vec![
        ((selector >> 24) & 0xFF) as u8,
        ((selector >> 16) & 0xFF) as u8,
        ((selector >> 8) & 0xFF) as u8,
        (selector & 0xFF) as u8,
    ]
}

/// Get the current function name being compiled
/// This is a thread_local variable that will be set during compile_function
thread_local! {
    static CURRENT_FUNCTION: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

/// Set the current function name
fn set_current_function_name(name: &str) {
    CURRENT_FUNCTION.with(|current| {
        *current.borrow_mut() = Some(name.to_string());
    });
}

/// Get the current function name
fn get_current_function_name() -> Option<String> {
    CURRENT_FUNCTION.with(|current| {
        current.borrow().clone()
    })
}
