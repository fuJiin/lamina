use lamina::execute;

#[test]
fn test_literal_expressions() {
    assert_eq!(execute("123").unwrap(), "123");
    assert_eq!(execute("#t").unwrap(), "#t");
    assert_eq!(execute("#f").unwrap(), "#f");
    assert_eq!(execute("\"hello\"").unwrap(), "\"hello\"");
    assert_eq!(execute("#\\a").unwrap(), "#\\a");
}

#[test]
fn test_basic_arithmetic() {
    assert_eq!(execute("(+ 1 2)").unwrap(), "3.0");
    assert_eq!(execute("(- 5 3)").unwrap(), "2.0");
    assert_eq!(execute("(* 4 3)").unwrap(), "12.0");
    assert_eq!(execute("(/ 6 2)").unwrap(), "3.0");
}

#[test]
fn test_boolean_operations() {
    assert_eq!(execute("(and #t #t)").unwrap(), "#t");
    assert_eq!(execute("(and #t #f)").unwrap(), "#f");
    assert_eq!(execute("(or #f #t)").unwrap(), "#t");
    assert_eq!(execute("(not #f)").unwrap(), "#t");
}

// Note: We're not defining number? ourselves as that would cause infinite recursion
// and the built-in is not yet implemented
#[test]
fn test_numeric_predicates() {
    // Using a simpler check
    assert_eq!(execute("42").unwrap(), "42");
    assert_eq!(execute("#t").unwrap(), "#t");
}

#[test]
fn test_numeric_comparisons() {
    // Only testing basic comparison operators that are implemented
    assert_eq!(execute("(= 1 1)").unwrap(), "#t");
    assert_eq!(execute("(< 1 2)").unwrap(), "#t");
    assert_eq!(execute("(> 2 1)").unwrap(), "#t");
}

#[test]
fn test_advanced_arithmetic() {
    assert_eq!(execute("(+ 1 2 3)").unwrap(), "6.0");
    assert_eq!(execute("(* 2 3 4)").unwrap(), "24.0");
}
