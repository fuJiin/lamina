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
    
    /// Comment for generated code
    Comment(String),
}

/// Represents a Huff macro definition
#[derive(Debug, Clone)]
pub struct HuffMacro {
    pub name: String,
    pub takes: usize,  // Number of stack inputs
    pub returns: usize, // Number of stack outputs
    pub instructions: Vec<Instruction>,
}

impl fmt::Display for HuffMacro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "#define macro {}_MACRO() = takes({}) returns({}) {{", 
                self.name.to_uppercase(), self.takes, self.returns)?;
        
        for instruction in &self.instructions {
            match instruction {
                Instruction::Simple(op) => writeln!(f, "    {}", op.as_huff_str())?,
                Instruction::Push(size, bytes) => {
                    let hex_str = bytes.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    writeln!(f, "    PUSH{} 0x{}", size, hex_str)?
                },
                Instruction::Label(label) => writeln!(f, "{}:", label)?,
                Instruction::JumpTo(label) => {
                    writeln!(f, "    // Jump to {}", label)?;
                    writeln!(f, "    __JUMPLABEL_{}", label)?
                },
                Instruction::JumpToIf(label) => {
                    writeln!(f, "    // Jump to {} if condition is met", label)?;
                    writeln!(f, "    __JUMPLABEL_{}", label)?;
                    writeln!(f, "    JUMPI")?
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
        
        // Define the Huff contract structure
        writeln!(f, "#define function owner() view returns (address)")?;
        writeln!(f, "\n#define macro MAIN() = takes(0) returns(0) {{")?;
        writeln!(f, "    // Get the function selector")?;
        writeln!(f, "    0x00 calldataload 0xe0 shr")?;
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