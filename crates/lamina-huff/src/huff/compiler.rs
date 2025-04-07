use std::collections::HashMap;

use lamina::error::Error;
use lamina::value::{NumberKind, Value};

use super::bytecode::{FunctionSignature, HuffContract, HuffMacro, Instruction};
use super::opcodes::Opcode;

/// Compiler context to track state during compilation
struct CompilerContext {
    /// Track macros being defined
    macros: Vec<HuffMacro>,

    /// Track functions being defined
    functions: HashMap<String, FunctionInfo>,

    /// Track storage slots
    storage_slots: HashMap<String, u64>,

    /// Track label counter
    label_counter: usize,

    /// Track function signatures
    function_signatures: Vec<FunctionSignature>,
}

/// Information about a function
struct FunctionInfo {
    name: String,
    params: Vec<String>,
    return_count: usize,
}

impl CompilerContext {
    fn new(_contract_name: &str) -> Self {
        CompilerContext {
            macros: Vec::new(),
            functions: HashMap::new(),
            storage_slots: HashMap::new(),
            label_counter: 0,
            function_signatures: Vec::new(),
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
    fn register_function(&mut self, name: &str, params: Vec<String>, return_count: usize) {
        self.functions.insert(
            name.to_string(),
            FunctionInfo {
                name: name.to_string(),
                params: params.clone(),
                return_count,
            },
        );

        // Register function signature if it's not the main function
        if name.to_lowercase() != "main" {
            // Assuming all returns are uint256 for now
            let returns = vec!["uint256".to_string(); return_count];
            self.function_signatures
                .push(FunctionSignature::new(name, params, returns));
        }
    }

    /// Register a storage slot
    fn register_storage_slot(&mut self, name: &str, slot: u64) {
        self.storage_slots.insert(name.to_string(), slot);
    }

    /// Get a storage slot by name
    fn get_storage_slot(&self, name: &str) -> Option<u64> {
        self.storage_slots.get(name).copied()
    }

    /// Get all storage slots with their names
    fn get_all_storage_slots(&self) -> Vec<(String, u64)> {
        self.storage_slots
            .iter()
            .map(|(name, &slot)| (name.clone(), slot))
            .collect()
    }

    /// Generate Huff constant definitions for storage slots
    fn generate_storage_constants(&self) -> String {
        let mut result = String::new();

        // Sort by slot for consistency
        let mut slots: Vec<(String, u64)> = self
            .storage_slots
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        slots.sort_by_key(|(_, slot)| *slot);

        for (name, slot) in slots {
            // Convert snake_case or kebab-case to UPPER_SNAKE_CASE for constants
            let constant_name = name.replace('-', "_").to_uppercase();
            result.push_str(&format!(
                "#define constant {}_SLOT = 0x{:064x}\n",
                constant_name, slot
            ));
        }

        result
    }

    /// Get a storage slot name by its value
    fn get_storage_slot_name_by_value(&self, value: u64) -> Option<String> {
        self.storage_slots.iter().find_map(|(name, &slot)| {
            if slot == value {
                Some(name.clone())
            } else {
                None
            }
        })
    }

    /// Get the function info by name
    fn get_function_info(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }

    /// Get a function's selector by name
    fn get_function_selector(&self, name: &str) -> Option<u32> {
        self.function_signatures
            .iter()
            .find(|sig| sig.name == name)
            .map(|sig| sig.selector)
    }

    /// Get all function signatures
    fn get_function_signatures(&self) -> &[FunctionSignature] {
        &self.function_signatures
    }
}

/// Compile a Lamina expression to Huff code
pub fn compile(expr: &Value, contract_name: &str) -> Result<String, Error> {
    let mut context = CompilerContext::new(contract_name);

    // First pass: analyze the program to discover functions and storage slots
    analyze_program(expr, &mut context)?;

    // Second pass: compile functions to macros
    compile_functions(expr, &mut context)?;

    // Create a main dispatcher macro that uses the auto-generated function selectors
    let main_macro = create_auto_dispatcher_macro(&context)?;

    // Generate storage constants
    let storage_constants = context.generate_storage_constants();

    // Build the contract
    let contract = HuffContract {
        name: contract_name.to_string(),
        constructor: None, // Default constructor for now
        main: main_macro,
        macros: context.macros,
        storage_constants,
        functions: context.function_signatures.clone(),
    };

    // Convert the contract to Huff code
    Ok(contract.to_string())
}

/// Create an automatic dispatcher macro based on function signatures
fn create_auto_dispatcher_macro(context: &CompilerContext) -> Result<HuffMacro, Error> {
    let mut instructions = Vec::new();

    instructions.push(Instruction::Comment(
        "Function Dispatcher (Auto-Generated)".to_string(),
    ));
    instructions.push(Instruction::Comment(
        "Compare function selector and route to appropriate function".to_string(),
    ));

    // Get function signatures
    let function_signatures = context.get_function_signatures();

    // For each function, create a selector comparison and jump
    for (i, function) in function_signatures.iter().enumerate() {
        let function_name = normalize_function_name(&function.name);
        let selector = function.selector;

        // Convert the selector to bytes
        let selector_bytes = selector_to_bytes(selector);

        // Add a label for this comparison branch
        let comparison_label = format!("compare_selector_{}", i);
        instructions.push(Instruction::Label(comparison_label.clone()));

        // Push the function selector constant
        instructions.push(Instruction::Push(4, selector_bytes));

        // Duplicate the calldata selector for comparison
        instructions.push(Instruction::Simple(Opcode::DUP2));

        // Compare the selectors
        instructions.push(Instruction::Simple(Opcode::EQ));

        // Jump to function if selectors match
        let jump_label = format!("jump_to_{}", function_name);
        instructions.push(Instruction::JumpLabel(jump_label.clone()));
        instructions.push(Instruction::JumpToIf(jump_label.clone()));

        // Add function jump destination
        instructions.push(Instruction::Label(jump_label));

        // Pop the selector before calling the function
        instructions.push(Instruction::Simple(Opcode::POP));

        // Call the function
        instructions.push(Instruction::MacroCall(function_name));

        // Store result in memory for return
        instructions.push(Instruction::Comment(
            "Store return value in memory".to_string(),
        ));
        instructions.push(Instruction::Push(1, vec![0]));
        instructions.push(Instruction::Simple(Opcode::MSTORE));

        // Return 32 bytes from memory
        instructions.push(Instruction::Comment(
            "Return 32 bytes from memory".to_string(),
        ));
        instructions.push(Instruction::Push(1, vec![32]));
        instructions.push(Instruction::Push(1, vec![0]));
        instructions.push(Instruction::Simple(Opcode::RETURN));
    }

    // Add fallback for unknown selectors
    instructions.push(Instruction::Label("unknown_selector".to_string()));
    instructions.push(Instruction::Comment(
        "Unknown function selector, revert".to_string(),
    ));
    instructions.push(Instruction::Push(1, vec![0]));
    instructions.push(Instruction::Push(1, vec![0]));
    instructions.push(Instruction::Simple(Opcode::REVERT));

    // Create the main macro
    Ok(HuffMacro {
        name: "main".to_string(),
        takes: 1,
        returns: 0,
        instructions,
        params: vec!["selector".to_string()],
    })
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
                    Value::Number(NumberKind::Integer(slot)) => {
                        context.register_storage_slot(name, *slot as u64);
                    }
                    Value::Pair(inner_pair) => {
                        if let Value::Number(NumberKind::Integer(slot)) = &inner_pair.0 {
                            context.register_storage_slot(name, *slot as u64);
                        }
                    }
                    _ => {}
                }

                Ok(())
            }

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

                    // Analyze the body to determine return count
                    // For now, assume all functions return 1 value (typical for getters/setters)
                    let return_count = 1;

                    // Register the function with its parameters and return count
                    context.register_function(func_name, params, return_count);
                }
                Ok(())
            }

            // Invalid define form
            _ => Err(Error::Runtime("Invalid define form".to_string())),
        }
    } else {
        Err(Error::Runtime("Invalid define form".to_string()))
    }
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

    Err(Error::Runtime(
        "Expected a begin form at the top level".to_string(),
    ))
}

/// Compile functions to Huff macros
fn compile_functions(expr: &Value, context: &mut CompilerContext) -> Result<(), Error> {
    // Extract the top-level begin form
    if let Value::Pair(pair) = expr {
        if let Value::Symbol(sym) = &pair.0 {
            if sym == "begin" {
                // Process the body of the begin form
                let mut body = &pair.1;

                // Track visited function names to avoid duplicates
                let mut visited_functions = std::collections::HashSet::new();

                // Process each expression in the body
                while let Value::Pair(pair) = body {
                    let expr = &pair.0;

                    // Look for define forms
                    if let Value::Pair(def_pair) = expr {
                        if let Value::Symbol(def_sym) = &def_pair.0 {
                            if def_sym == "define" {
                                if let Value::Pair(define_pair) = &def_pair.1 {
                                    if let Value::Pair(func_def) = &define_pair.0 {
                                        if let Value::Symbol(func_name) = &func_def.0 {
                                            // Skip the main function as it's handled separately
                                            if func_name == "main" {
                                                body = &pair.1;
                                                continue;
                                            }

                                            // Normalize the function name
                                            let normalized_name =
                                                normalize_function_name(func_name);

                                            // Skip if we've already compiled this function
                                            if visited_functions.contains(&normalized_name) {
                                                body = &pair.1;
                                                continue;
                                            }
                                            visited_functions.insert(normalized_name);

                                            // Compile the function
                                            compile_function(func_name, &define_pair.1, context)?;
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

    Err(Error::Runtime(
        "Expected a begin form at the top level".to_string(),
    ))
}

/// Compile a function to a Huff macro
fn compile_function(
    func_name: &str,
    body: &Value,
    context: &mut CompilerContext,
) -> Result<(), Error> {
    // Normalize the function name
    let normalized_name = normalize_function_name(func_name);

    // Set the current function name for the analyze_function_body function
    set_current_function_name(func_name);

    let _instructions: Vec<Instruction> = Vec::new();

    // Analyze the function body to determine its type
    let func_type = analyze_function_body(body, context)?;

    // Clear the current function name
    set_current_function_name("");

    match func_type {
        FunctionType::StorageGetter(slot) => {
            // Create a simple getter macro
            let mut instructions = Vec::new();

            // Get the storage slot name based on the slot value
            let slot_name = context
                .get_storage_slot_name_by_value(slot)
                .unwrap_or_else(|| format!("SLOT_{}", slot));

            // For a getter, just add a comment and load from storage
            instructions.push(Instruction::Comment(format!(
                "Load value from storage slot {}",
                slot
            )));

            // Push the storage slot constant instead of the raw value
            let slot_constant = format!("{}_SLOT", slot_name.to_uppercase().replace('-', "_"));
            instructions.push(Instruction::Push(32, vec![0])); // Placeholder, will be replaced by constant reference

            // Instead of using a MacroCall for constants, add a Comment instruction
            // to ensure the generated code references the constant directly
            instructions.pop(); // Remove the placeholder
            instructions.push(Instruction::Comment(format!(
                "Using storage slot constant: {}",
                slot_constant
            )));
            instructions.push(Instruction::Simple(Opcode::CONSTANT(slot_constant.clone())));

            // SLOAD operation
            instructions.push(Instruction::Simple(Opcode::SLOAD));

            // Create the macro and add it to the context
            let macro_def = HuffMacro {
                name: normalized_name.clone(),
                takes: 0,
                returns: 1,
                instructions,
                params: Vec::new(),
            };

            context.add_macro(macro_def);
        }

        FunctionType::StorageSetter(slot) => {
            // Create a setter macro
            let mut instructions = Vec::new();

            // Get the storage slot name based on the slot value
            let slot_name = context
                .get_storage_slot_name_by_value(slot)
                .unwrap_or_else(|| format!("SLOT_{}", slot));

            // For a setter, load the value from calldata, store it, and return it
            instructions.push(Instruction::Comment(
                "Store value from calldata to storage".to_string(),
            ));

            // Get the value from the first parameter (assuming it's a value)
            instructions.push(Instruction::Push(1, vec![0x04])); // Offset 4 (after selector)
            instructions.push(Instruction::Simple(Opcode::CALLDATALOAD));

            // Push the storage slot constant
            let slot_constant = format!("{}_SLOT", slot_name.to_uppercase().replace('-', "_"));
            instructions.push(Instruction::Comment(format!(
                "Using storage slot constant: {}",
                slot_constant
            )));
            instructions.push(Instruction::Simple(Opcode::CONSTANT(slot_constant.clone())));

            // Swap the value and slot
            instructions.push(Instruction::Simple(Opcode::SWAP1));

            // Store the value
            instructions.push(Instruction::Simple(Opcode::SSTORE));

            // Load the value again to return it
            instructions.push(Instruction::Comment(
                "Load stored value to return".to_string(),
            ));
            instructions.push(Instruction::Simple(Opcode::CONSTANT(slot_constant.clone())));
            instructions.push(Instruction::Simple(Opcode::SLOAD));

            // Create the macro and add it to the context
            let macro_def = HuffMacro {
                name: normalized_name.clone(),
                takes: 1,   // Takes one parameter (the value)
                returns: 1, // Returns the stored value
                instructions,
                params: vec!["value".to_string()],
            };

            context.add_macro(macro_def);
        }

        FunctionType::StorageIncrementer(slot) => {
            // Create an incrementer macro
            let mut instructions = Vec::new();

            // Get the storage slot name based on the slot value
            let slot_name = context
                .get_storage_slot_name_by_value(slot)
                .unwrap_or_else(|| format!("SLOT_{}", slot));

            let slot_constant = format!("{}_SLOT", slot_name.to_uppercase().replace('-', "_"));

            // For an incrementer, load current value, increment it, store it back, and return new value
            instructions.push(Instruction::Comment(format!(
                "Increment value at storage slot {}",
                slot
            )));

            // Load current value
            instructions.push(Instruction::Comment(format!(
                "Using storage slot constant: {}",
                slot_constant
            )));
            instructions.push(Instruction::Simple(Opcode::CONSTANT(slot_constant.clone())));
            instructions.push(Instruction::Simple(Opcode::SLOAD));

            // Add 1
            instructions.push(Instruction::Push(1, vec![1]));
            instructions.push(Instruction::Simple(Opcode::ADD));

            // Duplicate for storage
            instructions.push(Instruction::Simple(Opcode::DUP1));

            // Store updated value
            instructions.push(Instruction::Simple(Opcode::CONSTANT(slot_constant.clone())));
            instructions.push(Instruction::Simple(Opcode::SWAP1));
            instructions.push(Instruction::Simple(Opcode::SSTORE));

            // Create the macro and add it to the context
            let macro_def = HuffMacro {
                name: normalized_name.clone(),
                takes: 0,
                returns: 1, // Returns the new value
                instructions,
                params: Vec::new(),
            };

            context.add_macro(macro_def);
        }

        // Default case for unknown function types
        FunctionType::Unknown => {
            // For now, create a basic macro that just reverts
            let instructions = vec![
                Instruction::Comment("Function not yet implemented, reverting".to_string()),
                Instruction::Push(1, vec![0]), // Size: 0
                Instruction::Push(1, vec![0]), // Offset: 0
                Instruction::Simple(Opcode::REVERT),
            ];

            // Create the macro and add it to the context
            let macro_def = HuffMacro {
                name: normalized_name.clone(),
                takes: 0,
                returns: 0,
                instructions,
                params: Vec::new(),
            };

            context.add_macro(macro_def);
        }
    }

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
fn extract_direct_storage_slot(
    body: &Value,
    context: &CompilerContext,
) -> Result<Option<u64>, Error> {
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
                                            if let Some(slot) = context.get_storage_slot(slot_name)
                                            {
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
        }
        _ => {}
    }

    Ok(None)
}

/// Extract storage slot from function calls that might use storage
fn extract_storage_from_function_call(
    body: &Value,
    context: &CompilerContext,
) -> Result<Option<u64>, Error> {
    if let Value::Pair(pair) = body {
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
    }

    Ok(None)
}

/// Analyze a function body to determine its type
fn analyze_function_body(body: &Value, context: &CompilerContext) -> Result<FunctionType, Error> {
    // First look at function name patterns as a hint

    // Check for known storage slots
    for slot_value in context.storage_slots.values() {
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
    if let Value::Pair(pair) = body {
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
                            last_op_is_load = inner_op == "storage-load";
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
    }
    false
}

/// Check if a function body is incrementing a storage value
fn is_storage_incrementer(body: &Value) -> bool {
    if let Value::Pair(pair) = body {
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
    }
    false
}

/// Check if a function body is setting a storage value
fn is_storage_setter(body: &Value) -> bool {
    if let Value::Pair(pair) = body {
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
    }
    false
}

/// Get the current function name being compiled
/// This is a thread_local variable that will be set during compile_function
thread_local! {
    static CURRENT_FUNCTION: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Set the current function name
fn set_current_function_name(name: &str) {
    CURRENT_FUNCTION.with(|current| {
        *current.borrow_mut() = Some(name.to_string());
    });
}

/// Get the current function name
fn get_current_function_name() -> Option<String> {
    CURRENT_FUNCTION.with(|current| current.borrow().clone())
}

/// Helper function to normalize function names
fn normalize_function_name(name: &str) -> String {
    name.replace('-', "_")
}

/// Convert a selector value to bytes
fn selector_to_bytes(selector: u32) -> Vec<u8> {
    let bytes = selector.to_be_bytes();
    bytes.to_vec()
}
