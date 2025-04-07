use lamina::backends::huff;
use lamina::lexer;
use lamina::parser;

#[test]
fn test_compile_counter_contract() {
    // Counter contract Lamina code
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
          (define current (get-counter))
          (storage-store counter-slot (+ current 1))
          (get-counter)))
          
      ;; Handle function dispatch
      (define (main selector)
        (if (= selector 0x8ada066e) ;; "getCounter()"
            (get-counter)
            (if (= selector 0xd09de08a) ;; "increment()"
                (increment)
                (revert "Unknown function")))))
    "#;

    // Parse and evaluate the code
    let tokens = lexer::lex(lamina_code).unwrap();
    let expr = parser::parse(&tokens).unwrap();
    
    // Compile to Huff
    let huff_code = huff::compile(&expr, "Counter").unwrap();
    
    // Basic verification
    assert!(huff_code.contains("Counter"));
    assert!(huff_code.contains("get_counter"));
    assert!(huff_code.contains("increment"));
    assert!(huff_code.contains("SLOAD"));
    assert!(huff_code.contains("SSTORE"));
}

#[test]
fn test_compile_simple_storage() {
    // Simple storage contract Lamina code
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
          (get-value)))
          
      ;; Handle function dispatch
      (define (main selector)
        (if (= selector 0x6d4ce63c) ;; "getValue()"
            (get-value)
            (if (= selector 0x60fe47b1) ;; "setValue(uint256)"
                (set-value (get-calldata-word 1))
                (revert "Unknown function")))))
    "#;

    // Parse and evaluate the code
    let tokens = lexer::lex(lamina_code).unwrap();
    let expr = parser::parse(&tokens).unwrap();
    
    // Compile to Huff
    let huff_code = huff::compile(&expr, "SimpleStorage").unwrap();
    
    // Basic verification
    assert!(huff_code.contains("SimpleStorage"));
    assert!(huff_code.contains("get_value"));
    assert!(huff_code.contains("set_value"));
    assert!(huff_code.contains("CALLDATALOAD"));
    assert!(huff_code.contains("SLOAD"));
    assert!(huff_code.contains("SSTORE"));
} 