use lamina::execute;

#[test]
fn test_variable_references() {
    assert_eq!(execute("(define x 42)").unwrap(), "");
    assert_eq!(execute("x").unwrap(), "42");
}

#[test]
fn test_conditional_expressions() {
    assert_eq!(execute("(if #t 1 2)").unwrap(), "1");
    assert_eq!(execute("(if #f 1 2)").unwrap(), "2");
    assert_eq!(execute("(cond ((= 1 2) 3) (else 4))").unwrap(), "4");
}

#[test]
fn test_assignment_expressions() {
    assert_eq!(execute("(define x 1)").unwrap(), "");
    assert_eq!(execute("(set! x 2)").unwrap(), "");
    assert_eq!(execute("x").unwrap(), "2");
}

#[test]
fn test_let_expressions() {
    assert_eq!(execute("(let ((x 1) (y 2)) (+ x y))").unwrap(), "3.0");
}

#[test]
fn test_let_star_expressions() {
    assert_eq!(
        execute("(let* ((x 1) (y (+ x 1))) (+ x y))").unwrap(),
        "3.0"
    );
}

#[test]
fn test_letrec_expressions() {
    assert_eq!(execute("(letrec ((x 1) (y 2)) (+ x y))").unwrap(), "3.0");
}

#[test]
fn test_quoting() {
    assert_eq!(execute("'123").unwrap(), "123");
    assert_eq!(execute("'(1 2 3)").unwrap(), "(1 2 3)");
    assert_eq!(execute("(quote hello)").unwrap(), "hello");
}
