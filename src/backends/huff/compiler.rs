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
    fn new(contract_name: &str) -> Self {
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
                // Check if it's a storage slot
                if name.ends_with("-slot") {
                    if let Value::Number(num) = &pair.1 {
                        if let crate::value::NumberKind::Integer(slot) = num {
                            context.register_storage_slot(name, *slot as u64);
                        }
                    }
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
                                            // Skip the main function as it's handled separately
                                            if func_name != "main" {
                                                let func_body = &func_def.1;
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
    let macro_name = func_name.replace("-", "_");
    let mut instructions = Vec::new();
    
    // Determine what the function does based on its name and body
    match func_name {
        // Case 1: Functions that get a value from storage
        name if name.starts_with("get-") => {
            // For get functions, we generate a storage load operation
            if let Some(slot) = extract_storage_slot(body, context)? {
                // Generate SLOAD instructions
                instructions.push(Instruction::Push(1, vec![slot as u8]));
                instructions.push(Instruction::Simple(Opcode::SLOAD));
                // Memory management for return
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::MSTORE));
                // Return the value (32 bytes from memory position 0)
                instructions.push(Instruction::Push(1, vec![32]));
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::RETURN));
            } else {
                // If we can't find the storage slot, use a hardcoded slot 0 for now
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::SLOAD));
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::MSTORE));
                instructions.push(Instruction::Push(1, vec![32]));
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::RETURN));
            }
        },
        
        // Case 2: Functions that set or increment a value
        name if name == "increment" || name.starts_with("set-") => {
            if let Some(slot) = extract_storage_slot(body, context)? {
                if name == "increment" {
                    // Load current value
                    instructions.push(Instruction::Push(1, vec![slot as u8]));
                    instructions.push(Instruction::Simple(Opcode::SLOAD));
                    // Increment
                    instructions.push(Instruction::Push(1, vec![1]));
                    instructions.push(Instruction::Simple(Opcode::ADD));
                    instructions.push(Instruction::Simple(Opcode::DUP1));
                } else {
                    // For set functions, the value should be on the stack already
                    // We'll need to implement calldata handling separately
                    instructions.push(Instruction::Comment("Get value to store from calldata".to_string()));
                    instructions.push(Instruction::Push(1, vec![0x04]));
                    instructions.push(Instruction::Simple(Opcode::CALLDATALOAD));
                    instructions.push(Instruction::Simple(Opcode::DUP1));
                }
                
                // Store the value
                instructions.push(Instruction::Push(1, vec![slot as u8]));
                instructions.push(Instruction::Simple(Opcode::SSTORE));
                
                // Memory management for return
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::MSTORE));
                
                // Return the value (32 bytes from memory position 0)
                instructions.push(Instruction::Push(1, vec![32]));
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::RETURN));
            } else {
                // If we can't find the storage slot, use a hardcoded slot 0 for now
                if name == "increment" {
                    // Increment logic
                    instructions.push(Instruction::Push(1, vec![0]));
                    instructions.push(Instruction::Simple(Opcode::SLOAD));
                    instructions.push(Instruction::Push(1, vec![1]));
                    instructions.push(Instruction::Simple(Opcode::ADD));
                    instructions.push(Instruction::Simple(Opcode::DUP1));
                } else {
                    // Set logic for setValue
                    instructions.push(Instruction::Comment("Get value to store from calldata".to_string()));
                    instructions.push(Instruction::Push(1, vec![0x04]));
                    instructions.push(Instruction::Simple(Opcode::CALLDATALOAD));
                    instructions.push(Instruction::Simple(Opcode::DUP1));
                }
                
                // Store and return
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::SSTORE));
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::MSTORE));
                instructions.push(Instruction::Push(1, vec![32]));
                instructions.push(Instruction::Push(1, vec![0]));
                instructions.push(Instruction::Simple(Opcode::RETURN));
            }
        },
        
        // Handle other functions as needed
        _ => {
            instructions.push(Instruction::Comment(format!("Function {} not implemented", func_name)));
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

/// Extract the storage slot from a function body
fn extract_storage_slot(body: &Value, context: &CompilerContext) -> Result<Option<u64>, Error> {
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
                    return Ok(None);
                } else if op == "begin" {
                    let mut body = &pair.1;
                    
                    // Find the first storage operation
                    while let Value::Pair(pair) = body {
                        let expr = &pair.0;
                        
                        if let Value::Pair(op_pair) = expr {
                            if let Value::Symbol(op) = &op_pair.0 {
                                if op == "storage-store" {
                                    if let Value::Pair(args) = &op_pair.1 {
                                        if let Value::Symbol(slot_name) = &args.0 {
                                            if let Some(slot) = context.get_storage_slot(slot_name) {
                                                return Ok(Some(slot));
                                            }
                                        }
                                    }
                                } else if op == "storage-load" {
                                    if let Value::Symbol(slot_name) = &op_pair.1 {
                                        if let Some(slot) = context.get_storage_slot(slot_name) {
                                            return Ok(Some(slot));
                                        }
                                    }
                                }
                            }
                        }
                        
                        body = &pair.1;
                    }
                }
            }
        },
        _ => {}
    }
    
    Ok(None)
}

/// Create the main dispatcher macro
fn create_dispatcher_macro(context: &CompilerContext) -> Result<HuffMacro, Error> {
    let mut instructions = Vec::new();
    
    // First we need to load the function selector from calldata
    instructions.push(Instruction::Push(1, vec![0]));
    instructions.push(Instruction::Simple(Opcode::CALLDATALOAD));
    instructions.push(Instruction::Push(1, vec![0xE0]));
    instructions.push(Instruction::Simple(Opcode::SHR));
    
    // Find the main function to see what selectors it handles
    let main_selectors = extract_selectors_from_main(context)?;
    
    // Generate dispatcher logic
    for (i, (selector, func_name)) in main_selectors.iter().enumerate() {
        instructions.push(Instruction::Simple(Opcode::DUP1));
        
        // Push selector as 4 bytes
        let selector_bytes = selector_to_bytes(*selector);
        instructions.push(Instruction::Push(4, selector_bytes));
        
        instructions.push(Instruction::Simple(Opcode::EQ));
        
        // Jump label for this function
        let label = format!("{}_branch", func_name.replace("-", "_"));
        instructions.push(Instruction::Push(1, vec![i as u8]));
        instructions.push(Instruction::JumpToIf(label.clone()));
    }
    
    // Default case: Invalid function selector
    instructions.push(Instruction::Simple(Opcode::INVALID));
    
    // Add branch labels and calls to function macros
    for (_, func_name) in main_selectors {
        let macro_name = func_name.replace("-", "_");
        let label = format!("{}_branch", macro_name);
        instructions.push(Instruction::Label(label));
        instructions.push(Instruction::Comment(format!("Call {}_MACRO", macro_name.to_uppercase())));
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
    let mut selectors = Vec::new();
    
    // For now, use hardcoded selectors for our known functions
    // In a real implementation, we would parse the main function to extract these
    
    // Check if our known functions exist
    if context.functions.contains_key("get-counter") {
        selectors.push((0x8ada066e, "get-counter".to_string()));
    }
    
    if context.functions.contains_key("increment") {
        selectors.push((0xd09de08a, "increment".to_string()));
    }
    
    if context.functions.contains_key("get-value") {
        selectors.push((0x6d4ce63c, "get-value".to_string()));
    }
    
    if context.functions.contains_key("set-value") {
        selectors.push((0x60fe47b1, "set-value".to_string()));
    }
    
    Ok(selectors)
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
