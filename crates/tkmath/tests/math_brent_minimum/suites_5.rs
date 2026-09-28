#[test]
fn MathBrentMinimumTest__MultipleCalls() {
    // Test multiple calls to Perform with same instance
    let mut aFunc1 = QuadraticFunction;
    let mut aFunc2 = CosineFunction;

    let mut aSolver = BrentMinimum::new(1.0e-10);

    // First call
    aSolver.perform(&mut aFunc1, 0.0, 1.5, 4.0);
    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap(), 2.0, 1.0e-8);

    // Second call with different function
    aSolver.perform(&mut aFunc2, 2.5, 3.1, 4.0);
    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap(), std::f64::consts::PI, 1.0e-8);
}

#[test]
fn MathBrentMinimumTest__EdgeCaseAtBoundary() {
    // Test when minimum is very close to one of the boundaries
    let mut aFunc = QuadraticFunction; // Minimum at x = 2

    let mut aSolver = BrentMinimum::new(1.0e-10);
    aSolver.perform(&mut aFunc, 2.0, 2.1, 3.0); // Minimum at left boundary

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap(), 2.0, 1.0e-8);
}
