use lamina::execute;

#[test]
fn test_exception_handling() {
    // Basic exception raising and handling
    assert_eq!(
        execute("(with-exception-handler (lambda (ex) 'caught) (lambda () (raise 'error) 'not-reached))").unwrap(),
        "caught"
    );

    // Error procedure
    assert_eq!(
        execute("(with-exception-handler (lambda (ex) 'caught-error) (lambda () (error \"an error occurred\") 'not-reached))").unwrap(),
        "caught-error"
    );

    // Simple guard test - using just 'else' clause
    assert_eq!(
        execute("(guard (ex (else 'handled)) (raise 'my-error) 'not-reached)").unwrap(),
        "handled"
    );
}

#[test]
fn test_record_types() {
    // Define a simple point record type
    assert_eq!(
        execute("(define-record-type <point> (make-point x y) point? (x point-x) (y point-y))")
            .unwrap(),
        ""
    );

    // Create a point and access its fields
    assert_eq!(execute("(define p (make-point 3 4))").unwrap(), "");
    assert_eq!(execute("(point? p)").unwrap(), "#t");
    assert_eq!(execute("(point-x p)").unwrap(), "3");
    assert_eq!(execute("(point-y p)").unwrap(), "4");

    // Test predicate on non-points
    assert_eq!(execute("(point? 42)").unwrap(), "#f");
}

#[test]
fn test_bytevectors() {
    // Create bytevectors
    assert_eq!(execute("(define bv (bytevector 1 2 3 4))").unwrap(), "");
    assert_eq!(execute("(bytevector-length bv)").unwrap(), "4");

    // Access and update bytevector elements
    assert_eq!(execute("(bytevector-u8-ref bv 0)").unwrap(), "1");
    assert_eq!(execute("(bytevector-u8-set! bv 0 42)").unwrap(), "");
    assert_eq!(execute("(bytevector-u8-ref bv 0)").unwrap(), "42");

    // Convert between bytevectors and strings
    assert_eq!(execute("(define str \"ABC\")").unwrap(), "");
    assert_eq!(execute("(define bv2 (string->utf8 str))").unwrap(), "");
    assert_eq!(execute("(bytevector-length bv2)").unwrap(), "3");
    assert_eq!(execute("(utf8->string bv2)").unwrap(), "\"ABC\"");
}

#[test]
fn test_standard_procedures() {
    // String operations
    assert_eq!(
        execute("(string-map char-upcase \"hello\")").unwrap(),
        "\"HELLO\""
    );

    // For string-for-each, it returns (), which is displayed as empty string
    assert_eq!(execute("(define count 0)").unwrap(), "");
    assert_eq!(
        execute("(string-for-each (lambda (c) (set! count (+ count 1))) \"hello\")").unwrap(),
        ""
    );
    assert_eq!(execute("count").unwrap(), "5.0");

    // Vector operations - note that vectors are displayed as #(...) in Scheme
    assert_eq!(execute("(define v (vector 1 2 3))").unwrap(), "");
    assert_eq!(
        execute("(vector-map (lambda (x) (* x 2)) v)").unwrap(),
        "#(2.0 4.0 6.0)"
    );
    assert_eq!(execute("(define sum 0)").unwrap(), "");
    assert_eq!(
        execute("(vector-for-each (lambda (x) (set! sum (+ sum x))) v)").unwrap(),
        ""
    );
    assert_eq!(execute("sum").unwrap(), "6.0");

    // Numeric operations
    assert_eq!(execute("(exact-integer? 42)").unwrap(), "#t");
    assert_eq!(execute("(exact-integer? 42.0)").unwrap(), "#f");
    assert_eq!(execute("(exact? 42)").unwrap(), "#t");
    assert_eq!(execute("(inexact? 42.0)").unwrap(), "#t");
}
