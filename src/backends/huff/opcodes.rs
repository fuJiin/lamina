/// EVM Opcodes used in Huff
#[derive(Debug, Clone, Copy, PartialEq)]
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
}

impl Opcode {
    /// Converts an opcode to its string representation in Huff
    pub fn as_huff_str(&self) -> &'static str {
        match self {
            // Stack operations
            Opcode::PUSH0 => "0x00 PUSH0",
            Opcode::PUSH1 => "PUSH1",
            Opcode::PUSH2 => "PUSH2",
            Opcode::PUSH32 => "PUSH32",
            Opcode::POP => "POP",
            Opcode::DUP1 => "DUP1",
            Opcode::DUP2 => "DUP2",
            Opcode::DUP16 => "DUP16",
            Opcode::SWAP1 => "SWAP1",
            Opcode::SWAP2 => "SWAP2",
            Opcode::SWAP16 => "SWAP16",

            // Arithmetic operations
            Opcode::ADD => "ADD",
            Opcode::SUB => "SUB",
            Opcode::MUL => "MUL",
            Opcode::DIV => "DIV",
            Opcode::SDIV => "SDIV",
            Opcode::MOD => "MOD",
            Opcode::SMOD => "SMOD",
            Opcode::ADDMOD => "ADDMOD",
            Opcode::MULMOD => "MULMOD",
            Opcode::EXP => "EXP",

            // Comparison operations
            Opcode::LT => "LT",
            Opcode::GT => "GT",
            Opcode::SLT => "SLT",
            Opcode::SGT => "SGT",
            Opcode::EQ => "EQ",
            Opcode::ISZERO => "ISZERO",

            // Bitwise operations
            Opcode::AND => "AND",
            Opcode::OR => "OR",
            Opcode::XOR => "XOR",
            Opcode::NOT => "NOT",
            Opcode::SHL => "SHL",
            Opcode::SHR => "SHR",
            Opcode::SAR => "SAR",

            // Memory operations
            Opcode::MLOAD => "MLOAD",
            Opcode::MSTORE => "MSTORE",
            Opcode::MSTORE8 => "MSTORE8",
            Opcode::MSIZE => "MSIZE",

            // Storage operations
            Opcode::SLOAD => "SLOAD",
            Opcode::SSTORE => "SSTORE",

            // Program counter operations
            Opcode::JUMP => "JUMP",
            Opcode::JUMPI => "JUMPI",
            Opcode::PC => "PC",
            Opcode::JUMPDEST => "JUMPDEST",

            // Environment operations
            Opcode::ADDRESS => "ADDRESS",
            Opcode::BALANCE => "BALANCE",
            Opcode::ORIGIN => "ORIGIN",
            Opcode::CALLER => "CALLER",
            Opcode::CALLVALUE => "CALLVALUE",
            Opcode::CALLDATALOAD => "CALLDATALOAD",
            Opcode::CALLDATASIZE => "CALLDATASIZE",
            Opcode::CALLDATACOPY => "CALLDATACOPY",
            Opcode::CODESIZE => "CODESIZE",
            Opcode::CODECOPY => "CODECOPY",
            Opcode::GASPRICE => "GASPRICE",
            Opcode::EXTCODESIZE => "EXTCODESIZE",
            Opcode::EXTCODECOPY => "EXTCODECOPY",
            Opcode::RETURNDATASIZE => "RETURNDATASIZE",
            Opcode::RETURNDATACOPY => "RETURNDATACOPY",
            Opcode::EXTCODEHASH => "EXTCODEHASH",

            // Block operations
            Opcode::BLOCKHASH => "BLOCKHASH",
            Opcode::COINBASE => "COINBASE",
            Opcode::TIMESTAMP => "TIMESTAMP",
            Opcode::NUMBER => "NUMBER",
            Opcode::DIFFICULTY => "DIFFICULTY",
            Opcode::GASLIMIT => "GASLIMIT",
            Opcode::CHAINID => "CHAINID",
            Opcode::SELFBALANCE => "SELFBALANCE",
            Opcode::BASEFEE => "BASEFEE",

            // Control flow operations
            Opcode::STOP => "STOP",
            Opcode::RETURN => "RETURN",
            Opcode::REVERT => "REVERT",
            Opcode::INVALID => "INVALID",
            Opcode::SELFDESTRUCT => "SELFDESTRUCT",

            // Call operations
            Opcode::CALL => "CALL",
            Opcode::CALLCODE => "CALLCODE",
            Opcode::DELEGATECALL => "DELEGATECALL",
            Opcode::STATICCALL => "STATICCALL",
            Opcode::CREATE => "CREATE",
            Opcode::CREATE2 => "CREATE2",

            // Log operations
            Opcode::LOG0 => "LOG0",
            Opcode::LOG1 => "LOG1",
            Opcode::LOG2 => "LOG2",
            Opcode::LOG3 => "LOG3",
            Opcode::LOG4 => "LOG4",

            // Keccak
            Opcode::SHA3 => "SHA3",
        }
    }
}

/// Helper function to convert Opcode to Huff representation
pub fn to_huff(opcode: Opcode) -> &'static str {
    opcode.as_huff_str()
}
