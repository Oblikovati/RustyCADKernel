#[test]
fn MathBissecNewtonTest__IterationLimit() {
    // Test behavior with limited iterations
    let mut aFunc = CubicWithDerivative;

    let mut aSolver = BissecNewton::new(1.0e-12);
    aSolver.perform(&mut aFunc, 1.0, 2.0, 5); // Very few iterations

    // May or may not converge with so few iterations
    if aSolver.is_done() {
        assert_near(aSolver.value().unwrap(), 0.0, 1.0e-3);
    }
}

#[test]
fn MathBissecNewtonTest__InvalidBounds() {
    // Test with bounds that don't bracket a root
    let mut aFunc = QuadraticWithDerivative;

    let mut aSolver = BissecNewton::new(1.0e-10);
    aSolver.perform(&mut aFunc, 1.5, 2.5, 100); // Both bounds give positive function values

    // Function may fail to find root or find the nearby root at x=3
    if aSolver.is_done() {
        let aRoot: f64 = aSolver.root().unwrap();
        assert!(aRoot > 1.4);
        assert!(aRoot < 3.1);
    }
}

#[test]
fn MathBissecNewtonTest__ZeroDerivativeHandling() {
    // Test function where derivative can be zero (f(x) = x^3 has f'(0) = 0)
    let mut aFunc = CubicZeroDerivative;

    let mut aSolver = BissecNewton::new(1.0e-10);
    aSolver.perform(&mut aFunc, -0.5, 0.5, 100); // Root at x = 0

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 0.0, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
    assert_near(aSolver.derivative().unwrap(), 0.0, 1.0e-8);
}
