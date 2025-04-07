use std::fmt;

/// Represents an Ethereum address (20 bytes)
#[derive(Debug, Clone, PartialEq)]
pub struct Address([u8; 20]);

impl Address {
    pub fn new(bytes: [u8; 20]) -> Self {
        Address(bytes)
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches("0x");
        if hex.len() != 40 {
            return Err("Invalid address length".to_string());
        }

        let mut bytes = [0u8; 20];
        for i in 0..20 {
            let byte_str = &hex[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(byte_str, 16)
                .map_err(|_| format!("Invalid hex character in address: {}", byte_str))?;
        }

        Ok(Address(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// Represents an Ethereum function signature
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub name: String,
    pub inputs: Vec<ParameterType>,
    pub outputs: Vec<ParameterType>,
    pub selector: [u8; 4],
}

impl FunctionSignature {
    pub fn new(name: &str, inputs: Vec<ParameterType>, outputs: Vec<ParameterType>) -> Self {
        // In a real implementation, this would calculate the actual Keccak-256 hash
        // For now, we'll use a placeholder implementation
        let selector = compute_selector(name, &inputs);

        FunctionSignature {
            name: name.to_string(),
            inputs,
            outputs,
            selector,
        }
    }

    pub fn signature_string(&self) -> String {
        let inputs_str = self
            .inputs
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        format!("{}({})", self.name, inputs_str)
    }
}

/// Computes the 4-byte function selector from name and parameter types
fn compute_selector(name: &str, inputs: &[ParameterType]) -> [u8; 4] {
    // In a real implementation, this would use keccak256 to hash the function signature
    // For this example, we'll just use a simple mock implementation
    let inputs_str = inputs
        .iter()
        .map(|p| p.to_solidity_string())
        .collect::<Vec<_>>()
        .join(",");

    let signature = format!("{}({})", name, inputs_str);

    // Mock implementation - in reality, you would use keccak256 and take the first 4 bytes
    let mut result = [0u8; 4];
    for (i, byte) in signature.bytes().take(4).enumerate() {
        if i < 4 {
            result[i] = byte;
        }
    }

    result
}

/// Represents Solidity parameter types
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterType {
    Address,
    Bool,
    Uint(usize),  // e.g., uint256
    Int(usize),   // e.g., int128
    Bytes(usize), // e.g., bytes32
    DynamicBytes,
    String,
    Array(Box<ParameterType>), // e.g., uint256[]
    Tuple(Vec<ParameterType>), // e.g., (uint256,address)
}

impl ParameterType {
    fn to_solidity_string(&self) -> String {
        match self {
            ParameterType::Address => "address".to_string(),
            ParameterType::Bool => "bool".to_string(),
            ParameterType::Uint(bits) => format!("uint{}", bits),
            ParameterType::Int(bits) => format!("int{}", bits),
            ParameterType::Bytes(size) => format!("bytes{}", size),
            ParameterType::DynamicBytes => "bytes".to_string(),
            ParameterType::String => "string".to_string(),
            ParameterType::Array(element_type) => {
                format!("{}[]", element_type.to_solidity_string())
            }
            ParameterType::Tuple(types) => {
                let types_str = types
                    .iter()
                    .map(|t| t.to_solidity_string())
                    .collect::<Vec<_>>()
                    .join(",");
                format!("({})", types_str)
            }
        }
    }
}

impl fmt::Display for ParameterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_solidity_string())
    }
}
