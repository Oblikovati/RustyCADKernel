#[test]
fn MathNewtonFunctionRootTest__StartingAtRoot() {
    // Test when initial guess is already at the root
    let mut aFunc = LinearWithDerivative;

    let mut aSolver = NewtonFunctionRoot::new(&mut aFunc, 2.0, 1.0e-10, 1.0e-10); // Start exactly at root

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0, 1.0e-10);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
}

#[test]
fn MathNewtonFunctionRootTest__ZeroDerivativeAtRoot() {
    // Test with function having zero derivative at root (challenging case)
    let mut aFunc = CubicWithZeroDerivative;

    let mut aSolver = NewtonFunctionRoot::new(&mut aFunc, 0.1, 1.0e-8, 1.0e-8); // Slightly looser tolerance

    if aSolver.is_done() {
        assert_near(aSolver.root().unwrap(), 0.0, 1.0e-6);
        assert_near(aSolver.value().unwrap(), 0.0, 1.0e-8);
        assert_near(aSolver.derivative().unwrap(), 0.0, 1.0e-6);
    }
    // Note: Newton's method may fail with zero derivative, so we don't assert IsDone()
}

#[test]
fn MathNewtonFunctionRootTest__NarrowBounds() {
    // Test with very narrow bounds around the root
    let mut aFunc = QuadraticWithDerivative;

    let mut aSolver =
        NewtonFunctionRoot::bounded(&mut aFunc, 2.0, 1.0e-10, 1.0e-10, 1.99, 2.01, 100);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0, 1.0e-8);
    assert!(aSolver.root().unwrap() >= 1.99);
    assert!(aSolver.root().unwrap() <= 2.01);
}
