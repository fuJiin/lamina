use lamina::backends::huff;
use lamina::backends::huff::bytecode::calculate_function_selector;
use lamina::lexer;
use lamina::parser;

// Calculate selectors for the tests
fn get_selector(name: &str, params: &[&str]) -> u32 {
    calculate_function_selector(name, params)
}

#[test]
fn test_compile_counter_contract() {
    // Calculate expected selectors
    let get_counter_selector = format!("0x{:08x}", get_selector("get-counter", &[]));
    let increment_selector = format!("0x{:08x}", get_selector("increment", &[]));

    // Counter contract Lamina code with automatic function dispatch
    let lamina_code = r#"
    (begin
      ;; Define a storage slot for our counter
      (define counter-slot 0)
      
      ;; Get the counter value
      (define (get-counter)
        (storage-load counter-slot))
      
      ;; Increment the counter
      (define (increment)
        (begin
          (define current (storage-load counter-slot))
          (storage-store counter-slot (+ current 1))
          (storage-load counter-slot)))
    )"#;

    // Parse and evaluate the code
    let tokens = lexer::lex(lamina_code).unwrap();
    let expr = parser::parse(&tokens).unwrap();

    // Compile to Huff
    let huff_code = huff::compile(&expr, "Counter").unwrap();

    // Basic verification
    assert!(huff_code.contains("Counter"));
    assert!(huff_code.contains("get_counter"));
    assert!(huff_code.contains("increment"));
    assert!(huff_code.contains("sload"));
    assert!(huff_code.contains("sstore"));

    // Verify automatic selector generation
    assert!(huff_code.contains("#define function getCounter() view returns (uint256)"));
    assert!(huff_code.contains("#define function increment() view returns (uint256)"));

    // Verify dispatcher logic is present
    assert!(huff_code.contains("Function Dispatcher (Auto-Generated)"));
    assert!(huff_code.contains("0x00 calldataload"));
    assert!(huff_code.contains("0xe0 shr"));

    // Use the calculated selectors
    assert!(huff_code.contains(&get_counter_selector)); // getCounter()
    assert!(huff_code.contains(&increment_selector)); // increment()
}

#[test]
fn test_compile_simple_storage() {
    // Calculate expected selectors
    let get_value_selector = format!("0x{:08x}", get_selector("get-value", &[]));
    let set_value_selector = format!("0x{:08x}", get_selector("set-value", &["new-value"]));

    // Simple storage contract Lamina code with automatic function dispatch
    let lamina_code = r#"
    (begin
      ;; Define storage slot
      (define value-slot 0)
      
      ;; Get the stored value
      (define (get-value)
        (storage-load value-slot))
      
      ;; Set a new value
      (define (set-value new-value)
        (begin
          (storage-store value-slot new-value)
          (storage-load value-slot)))
    )"#;

    // Parse and evaluate the code
    let tokens = lexer::lex(lamina_code).unwrap();
    let expr = parser::parse(&tokens).unwrap();

    // Compile to Huff
    let huff_code = huff::compile(&expr, "SimpleStorage").unwrap();

    // Basic verification
    assert!(huff_code.contains("SimpleStorage"));
    assert!(huff_code.contains("get_value"));
    assert!(huff_code.contains("set_value"));
    assert!(huff_code.contains("calldataload"));
    assert!(huff_code.contains("sload"));
    assert!(huff_code.contains("sstore"));

    // Verify automatic selector generation
    assert!(huff_code.contains("#define function getValue() view returns (uint256)"));
    assert!(huff_code.contains("#define function setValue(uint256) view returns (uint256)"));

    // Verify dispatcher logic is present
    assert!(huff_code.contains("Function Dispatcher (Auto-Generated)"));

    // Use the calculated selectors instead of hardcoded values
    assert!(huff_code.contains(&get_value_selector)); // getValue()
    assert!(huff_code.contains(&set_value_selector)); // setValue(uint256)
}
