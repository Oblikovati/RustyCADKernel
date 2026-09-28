#[test]
fn MathBissecNewtonTest__NotDoneState() {
    // Test state handling for incomplete calculations
    let mut aSolver = BissecNewton::new(1.0e-10);

    assert!(!aSolver.is_done());
}

#[test]
fn MathBissecNewtonTest__MultipleCalls() {
    // Test multiple calls to Perform with the same solver instance
    let mut aFunc = QuadraticWithDerivative;
    let mut aSolver = BissecNewton::new(1.0e-10);

    // First call
    aSolver.perform(&mut aFunc, 0.0, 1.5, 100);
    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 1.0, 1.0e-8);

    // Second call with different bounds
    aSolver.perform(&mut aFunc, 2.5, 4.0, 100);
    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 3.0, 1.0e-8);
}

#[test]
fn MathBissecNewtonTest__HighPrecisionRequirement() {
    // Test with extremely tight tolerance
    let mut aFunc = SineWithDerivative;
    let mut aSolver = BissecNewton::new(1.0e-15);

    aSolver.perform(&mut aFunc, 3.0, 3.5, 200);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), std::f64::consts::PI, 1.0e-12);
}
