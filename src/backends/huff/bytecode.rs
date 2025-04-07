use std::fmt;

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
                Instruction::Push(size, bytes) => {
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

/// Represents a Huff contract with its macros
#[derive(Debug, Clone)]
pub struct HuffContract {
    pub name: String,
    pub constructor: Option<HuffMacro>,
    pub main: HuffMacro,
    pub macros: Vec<HuffMacro>,
    pub storage_constants: String, // New field for storage constants
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

        for mac in &self.macros {
            // Convert macro names to function definitions
            // Format: macro_name -> functionName
            let func_name = macro_to_function_name(&mac.name);

            // Skip duplicates and skip the main function
            if seen_functions.contains(&func_name) || mac.name.to_lowercase() == "main" {
                continue;
            }
            seen_functions.insert(func_name.clone());

            // Simple return type detection - all functions return uint256 for now
            // In a real implementation, this would be determined by analyzing the function
            writeln!(f, "#define function {}() view returns (uint256)", func_name)?;
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
    let parts: Vec<&str> = macro_name.split(|c| c == '_' || c == '-').collect();
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
