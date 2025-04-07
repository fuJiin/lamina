/// EVM Opcodes used in Huff
#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    // Stack operations
    PUSH0,
    PUSH1,
    PUSH2,
    PUSH32,
    POP,
    DUP1,
    DUP2,
    DUP16,
    SWAP1,
    SWAP2,
    SWAP16,

    // Arithmetic operations
    ADD,
    SUB,
    MUL,
    DIV,
    SDIV,
    MOD,
    SMOD,
    ADDMOD,
    MULMOD,
    EXP,

    // Comparison operations
    LT,
    GT,
    SLT,
    SGT,
    EQ,
    ISZERO,

    // Bitwise operations
    AND,
    OR,
    XOR,
    NOT,
    SHL,
    SHR,
    SAR,

    // Memory operations
    MLOAD,
    MSTORE,
    MSTORE8,
    MSIZE,

    // Storage operations
    SLOAD,
    SSTORE,

    // Program counter operations
    JUMP,
    JUMPI,
    PC,
    JUMPDEST,

    // Environment operations
    ADDRESS,
    BALANCE,
    ORIGIN,
    CALLER,
    CALLVALUE,
    CALLDATALOAD,
    CALLDATASIZE,
    CALLDATACOPY,
    CODESIZE,
    CODECOPY,
    GASPRICE,
    EXTCODESIZE,
    EXTCODECOPY,
    RETURNDATASIZE,
    RETURNDATACOPY,
    EXTCODEHASH,

    // Block operations
    BLOCKHASH,
    COINBASE,
    TIMESTAMP,
    NUMBER,
    DIFFICULTY,
    GASLIMIT,
    CHAINID,
    SELFBALANCE,
    BASEFEE,

    // Control flow operations
    STOP,
    RETURN,
    REVERT,
    INVALID,
    SELFDESTRUCT,

    // Call operations
    CALL,
    CALLCODE,
    DELEGATECALL,
    STATICCALL,
    CREATE,
    CREATE2,

    // Log operations
    LOG0,
    LOG1,
    LOG2,
    LOG3,
    LOG4,

    // Keccak
    SHA3,

    // Special variant for Huff constants
    CONSTANT(String),
}

impl Opcode {
    /// Converts an opcode to its string representation in Huff
    pub fn as_huff_str(&self) -> String {
        match self {
            // Constants need to be referenced directly
            Opcode::CONSTANT(name) => name.clone(),

            // Default case for normal opcodes
            _ => {
                match self {
                    // Stack operations
                    Opcode::PUSH0 => "0x00",
                    Opcode::PUSH1 => "0x01",
                    Opcode::PUSH2 => "0x02",
                    Opcode::PUSH32 => "0x32",
                    Opcode::POP => "pop",
                    Opcode::DUP1 => "dup1",
                    Opcode::DUP2 => "dup2",
                    Opcode::DUP16 => "dup16",
                    Opcode::SWAP1 => "swap1",
                    Opcode::SWAP2 => "swap2",
                    Opcode::SWAP16 => "swap16",

                    // Arithmetic operations
                    Opcode::ADD => "add",
                    Opcode::SUB => "sub",
                    Opcode::MUL => "mul",
                    Opcode::DIV => "div",
                    Opcode::SDIV => "sdiv",
                    Opcode::MOD => "mod",
                    Opcode::SMOD => "smod",
                    Opcode::ADDMOD => "addmod",
                    Opcode::MULMOD => "mulmod",
                    Opcode::EXP => "exp",

                    // Comparison operations
                    Opcode::LT => "lt",
                    Opcode::GT => "gt",
                    Opcode::SLT => "slt",
                    Opcode::SGT => "sgt",
                    Opcode::EQ => "eq",
                    Opcode::ISZERO => "iszero",

                    // Bitwise operations
                    Opcode::AND => "and",
                    Opcode::OR => "or",
                    Opcode::XOR => "xor",
                    Opcode::NOT => "not",
                    Opcode::SHL => "shl",
                    Opcode::SHR => "shr",
                    Opcode::SAR => "sar",

                    // Memory operations
                    Opcode::MLOAD => "mload",
                    Opcode::MSTORE => "mstore",
                    Opcode::MSTORE8 => "mstore8",
                    Opcode::MSIZE => "msize",

                    // Storage operations
                    Opcode::SLOAD => "sload",
                    Opcode::SSTORE => "sstore",

                    // Program counter operations
                    Opcode::JUMP => "jump",
                    Opcode::JUMPI => "jumpi",
                    Opcode::PC => "pc",
                    Opcode::JUMPDEST => "jumpdest",

                    // Environment operations
                    Opcode::ADDRESS => "address",
                    Opcode::BALANCE => "balance",
                    Opcode::ORIGIN => "origin",
                    Opcode::CALLER => "caller",
                    Opcode::CALLVALUE => "callvalue",
                    Opcode::CALLDATALOAD => "calldataload",
                    Opcode::CALLDATASIZE => "calldatasize",
                    Opcode::CALLDATACOPY => "calldatacopy",
                    Opcode::CODESIZE => "codesize",
                    Opcode::CODECOPY => "codecopy",
                    Opcode::GASPRICE => "gasprice",
                    Opcode::EXTCODESIZE => "extcodesize",
                    Opcode::EXTCODECOPY => "extcodecopy",
                    Opcode::RETURNDATASIZE => "returndatasize",
                    Opcode::RETURNDATACOPY => "returndatacopy",
                    Opcode::EXTCODEHASH => "extcodehash",

                    // Block operations
                    Opcode::BLOCKHASH => "blockhash",
                    Opcode::COINBASE => "coinbase",
                    Opcode::TIMESTAMP => "timestamp",
                    Opcode::NUMBER => "number",
                    Opcode::DIFFICULTY => "difficulty",
                    Opcode::GASLIMIT => "gaslimit",
                    Opcode::CHAINID => "chainid",
                    Opcode::SELFBALANCE => "selfbalance",
                    Opcode::BASEFEE => "basefee",

                    // Control flow operations
                    Opcode::STOP => "stop",
                    Opcode::RETURN => "return",
                    Opcode::REVERT => "revert",
                    Opcode::INVALID => "invalid",
                    Opcode::SELFDESTRUCT => "selfdestruct",

                    // Call operations
                    Opcode::CALL => "call",
                    Opcode::CALLCODE => "callcode",
                    Opcode::DELEGATECALL => "delegatecall",
                    Opcode::STATICCALL => "staticcall",
                    Opcode::CREATE => "create",
                    Opcode::CREATE2 => "create2",

                    // Log operations
                    Opcode::LOG0 => "log0",
                    Opcode::LOG1 => "log1",
                    Opcode::LOG2 => "log2",
                    Opcode::LOG3 => "log3",
                    Opcode::LOG4 => "log4",

                    // Keccak
                    Opcode::SHA3 => "sha3",

                    // This case should be unreachable as we've handled CONSTANT above
                    Opcode::CONSTANT(_) => unreachable!(),
                }
                .to_string()
            }
        }
    }
}

/// Helper function to convert Opcode to Huff representation
pub fn to_huff(opcode: Opcode) -> String {
    opcode.as_huff_str()
}
