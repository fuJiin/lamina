use std::fmt;
use tiny_keccak::{Hasher, Keccak};

use super::opcodes::Opcode;

/// Represents an EVM instruction with its arguments
#[derive(Debug, Clone)]
pub enum Instruction {
    /// Simple opcode without arguments (e.g., ADD, MUL)
    Simple(Opcode),

    /// Push opcode with a value (e.g., PUSH1 0x80)
    Push(u8, Vec<u8>),

    /// Label definition for jumps
    Label(String),

    /// Jump to a label
    JumpTo(String),

    /// Conditional jump to a label
    JumpToIf(String),

    /// Jump label for jumpdest
    JumpLabel(String),

    /// Call to another macro
    MacroCall(String),

    /// Comment for generated code
    Comment(String),
}

/// Represents a Huff macro definition
#[derive(Debug, Clone)]
pub struct HuffMacro {
    pub name: String,
    pub takes: usize,   // Number of stack inputs
    pub returns: usize, // Number of stack outputs
    pub instructions: Vec<Instruction>,
    pub params: Vec<String>, // Parameter names for the function
}

impl fmt::Display for HuffMacro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "#define macro {}_MACRO() = takes({}) returns({}) {{",
            self.name.to_uppercase().replace('-', "_"),
            self.takes,
            self.returns
        )?;

        for instruction in &self.instructions {
            match instruction {
                Instruction::Simple(op) => writeln!(f, "    {}", op.as_huff_str())?,
                Instruction::Push(_size, bytes) => {
                    let hex_str = bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    // Huff uses lowercase for hex values and doesn't require PUSH prefix
                    writeln!(f, "    0x{} ", hex_str)?
                }
                Instruction::Label(label) => writeln!(f, "{}:", label)?,
                Instruction::JumpTo(label) => {
                    writeln!(f, "    // Jump to {}", label)?;
                    writeln!(f, "    [{}] jump", label)?
                }
                Instruction::JumpToIf(label) => {
                    writeln!(f, "    // Jump to {} if condition is met", label)?;
                    writeln!(f, "    [{}] jumpi", label)?
                }
                Instruction::JumpLabel(label) => {
                    // For jump labels, we need to generate a proper label reference
                    writeln!(f, "    [{}]", label)?;
                }
                Instruction::MacroCall(macro_name) => {
                    // Check if this is a reference to a storage slot constant
                    if macro_name.ends_with("_SLOT") {
                        writeln!(f, "    {}", macro_name)?;
                    } else {
                        // For function macro calls, add _MACRO suffix and uppercase
                        writeln!(
                            f,
                            "    {}_MACRO()",
                            macro_name.to_uppercase().replace('-', "_")
                        )?;
                    }
                }
                Instruction::Comment(comment) => writeln!(f, "    // {}", comment)?,
            }
        }

        writeln!(f, "}}")
    }
}

/// Represents a function signature
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<String>,
    pub returns: Vec<String>,
    pub selector: u32,
}

impl FunctionSignature {
    pub fn new(name: &str, params: Vec<String>, returns: Vec<String>) -> Self {
        // Convert param strings to string slices for function selector calculation
        let param_slices: Vec<&str> = params.iter().map(|s| s.as_str()).collect();

        let selector = calculate_function_selector(name, &param_slices);

        FunctionSignature {
            name: name.to_string(),
            params,
            returns,
            selector,
        }
    }

    pub fn format_as_huff(&self) -> String {
        let function_name = macro_to_function_name(&self.name);

        // Format parameters - for now assume all are uint256
        let param_types = if self.params.is_empty() {
            "".to_string()
        } else {
            "uint256"
                .repeat(self.params.len())
                .chars()
                .collect::<Vec<_>>()
                .chunks(7) // Length of "uint256"
                .map(|c| c.iter().collect::<String>())
                .collect::<Vec<_>>()
                .join(",")
        };

        // Format return types - for now assume all are uint256
        let return_types = if self.returns.is_empty() {
            "".to_string()
        } else {
            format!(
                "returns ({})",
                "uint256"
                    .repeat(self.returns.len())
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(7) // Length of "uint256"
                    .map(|c| c.iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };

        format!(
            "#define function {}({}) view {}",
            function_name, param_types, return_types
        )
    }
}

/// Represents a Huff contract with its macros
#[derive(Debug, Clone)]
pub struct HuffContract {
    pub name: String,
    pub constructor: Option<HuffMacro>,
    pub main: HuffMacro,
    pub macros: Vec<HuffMacro>,
    pub storage_constants: String,         // For storage constants
    pub functions: Vec<FunctionSignature>, // Function signatures with selectors
}

impl fmt::Display for HuffContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "/* Generated Huff Contract: {} */", self.name)?;
        writeln!(f, "\n// SPDX-License-Identifier: MIT")?;
        writeln!(f, "// Compiler: Lamina-to-Huff\n")?;

        // First define the storage slots as constants
        if !self.storage_constants.is_empty() {
            writeln!(f, "/* Storage Slots */")?;
            writeln!(f, "{}", self.storage_constants)?;
        }

        // Define the function interfaces with proper signatures
        writeln!(f, "/* Function Signatures */")?;

        // Use a HashSet to track function signatures we've already written
        let mut seen_functions = std::collections::HashSet::new();

        for function in &self.functions {
            let func_name = &function.name;

            // Skip duplicates and skip the main function
            if seen_functions.contains(func_name) || func_name.to_lowercase() == "main" {
                continue;
            }
            seen_functions.insert(func_name.clone());

            // Write function signature with proper selector
            writeln!(f, "{}", function.format_as_huff())?;
        }

        // Write all the macros with proper Huff syntax
        writeln!(f, "\n/* Function Implementations */")?;

        // Write user-defined functions first (excluding main)
        for mac in &self.macros {
            if mac.name.to_lowercase() != "main" {
                writeln!(f, "{}\n", mac)?;
            }
        }

        // Constructor if any
        if let Some(constructor) = &self.constructor {
            writeln!(f, "{}\n", constructor)?;
        }

        // Main macro is required - place at the end
        writeln!(f, "{}\n", self.main)?;

        // Define the Huff entrypoint macros
        writeln!(f, "#define macro MAIN() = takes(0) returns(0) {{")?;
        writeln!(f, "    // Parse function selector from calldata")?;
        writeln!(
            f,
            "    0x00 calldataload     // load the first 32 bytes of calldata"
        )?;
        writeln!(
            f,
            "    0xe0 shr              // shift right by 0xe0 (224) bits to get the selector"
        )?;
        writeln!(
            f,
            "    {}_MACRO()",
            self.main.name.to_uppercase().replace('-', "_")
        )?;
        writeln!(f, "}}")?;

        if let Some(constructor) = &self.constructor {
            writeln!(f, "\n#define macro CONSTRUCTOR() = takes(0) returns (0) {{")?;
            writeln!(
                f,
                "    {}_MACRO()",
                constructor.name.to_uppercase().replace('-', "_")
            )?;
            writeln!(f, "}}")
        } else {
            writeln!(f, "\n#define macro CONSTRUCTOR() = takes(0) returns (0) {{")?;
            writeln!(f, "    // Default empty constructor")?;
            writeln!(f, "}}")
        }
    }
}

/// Convert a macro name to a function name in camelCase
fn macro_to_function_name(macro_name: &str) -> String {
    // Convert snake_case or kebab-case to camelCase
    let parts: Vec<&str> = macro_name.split(['_', '-']).collect();
    if parts.is_empty() {
        return String::new();
    }

    let mut result = parts[0].to_string();
    for part in parts.iter().skip(1) {
        if !part.is_empty() {
            result.push_str(&part[0..1].to_uppercase());
            result.push_str(&part[1..]);
        }
    }

    result
}

/// Calculate a function selector from a function name
/// This uses the standard Ethereum ABI function selector calculation:
/// first 4 bytes of keccak256(function_signature)
pub fn calculate_function_selector(name: &str, params: &[&str]) -> u32 {
    // Convert from snake_case or kebab-case to camelCase for solidity-style function names
    let function_name = macro_to_function_name(name);

    // Construct the function signature string: name(type1,type2,...)
    let mut signature = function_name;
    signature.push('(');

    // For now, assume all params are uint256
    // In a real implementation, we would analyze the parameter types
    if !params.is_empty() {
        for _ in 0..params.len() - 1 {
            signature.push_str("uint256,");
        }
        signature.push_str("uint256");
    }

    signature.push(')');

    // Calculate keccak256 hash of the signature
    let mut keccak = Keccak::v256();
    let mut hash = [0u8; 32];
    keccak.update(signature.as_bytes());
    keccak.finalize(&mut hash);

    // Take first 4 bytes and convert to u32
    u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
}
