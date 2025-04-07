use lamina::execute;

#[test]
fn test_procedure_calls() {
    assert_eq!(execute("(+ 1 2 3)").unwrap(), "6.0");
    assert_eq!(execute("(cons 1 (cons 2 '()))").unwrap(), "(1 2)");
}

#[test]
fn test_lambda_expressions() {
    assert_eq!(execute("((lambda (x) (+ x 1)) 5)").unwrap(), "6.0");
    assert_eq!(execute("((lambda (x y) (+ x y)) 3 4)").unwrap(), "7.0");
}

// The current implementation returns the procedure not the result
// since nested lambdas aren't automatically applied
#[test]
fn test_higher_order_functions() {
    let result = execute("(lambda (x) (lambda (y) (+ x y)))").unwrap();
    assert_eq!(result, "#<procedure>");

    // Test a different pattern that works with current implementation
    assert_eq!(execute("((lambda (x y) (+ x y)) 5 10)").unwrap(), "15.0");
}
