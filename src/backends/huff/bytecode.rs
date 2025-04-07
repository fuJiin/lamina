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
            self.name.to_uppercase(),
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
                    writeln!(f, "    PUSH{} 0x{}", size, hex_str)?
                }
                Instruction::Label(label) => writeln!(f, "{}:", label)?,
                Instruction::JumpTo(label) => {
                    writeln!(f, "    // Jump to {}", label)?;
                    writeln!(f, "    JUMP")?;
                }
                Instruction::JumpToIf(label) => {
                    writeln!(f, "    // Jump to {} if condition is met", label)?;
                    writeln!(f, "    JUMPI")?;
                }
                Instruction::JumpLabel(label) => {
                    // For jump labels, we need to generate a proper label reference
                    writeln!(f, "    PUSH1 [{}]", label)?;
                },
                Instruction::MacroCall(macro_name) => {
                    // For macro calls, just reference the macro directly
                    writeln!(f, "    {}", macro_name)?;
                },
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
}

impl fmt::Display for HuffContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "/* Generated Huff Contract: {} */", self.name)?;
        writeln!(f, "\n// SPDX-License-Identifier: MIT")?;
        writeln!(f, "// Compiler: Lamina-to-Huff\n")?;

        // Write all the macros
        for mac in &self.macros {
            writeln!(f, "{}\n", mac)?;
        }

        // Constructor if any
        if let Some(constructor) = &self.constructor {
            writeln!(f, "{}\n", constructor)?;
        }

        // Main macro is required
        writeln!(f, "{}\n", self.main)?;

        // Define the Huff contract functions based on the available macros
        for mac in &self.macros {
            // Convert macro names to function definitions
            // Format: macro_name -> functionName
            let func_name = macro_to_function_name(&mac.name);
            
            // Simple return type detection - all functions return uint256 for now
            // In a real implementation, this would be determined by analyzing the function
            writeln!(f, "#define function {}() view returns (uint256)", func_name)?;
        }
        
        writeln!(f, "\n#define macro MAIN() = takes(0) returns(0) {{")?;
        writeln!(f, "    {}_MACRO()", self.main.name.to_uppercase())?;
        writeln!(f, "}}")?;

        if let Some(constructor) = &self.constructor {
            writeln!(f, "\n#define macro CONSTRUCTOR() = takes(0) returns (0) {{")?;
            writeln!(f, "    {}_MACRO()", constructor.name.to_uppercase())?;
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
