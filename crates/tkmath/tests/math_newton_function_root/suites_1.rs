#[test]
fn MathNewtonFunctionRootTest__CubicRootFinding() {
    // Test finding root of cubic function
    let mut aFunc = CubicWithDerivative;

    let mut aSolver = NewtonFunctionRoot::new(&mut aFunc, 1.1, 1.0e-10, 1.0e-10); // Guess closer to root at 1

    assert!(aSolver.is_done());
    // Newton's method may converge to any of the roots (1, 2, or 3) depending on initial guess
    let aRoot: f64 = aSolver.root().unwrap();
    let aFoundValidRoot: bool = ((aRoot - 1.0).abs() < 1.0e-6)
        || ((aRoot - 2.0).abs() < 1.0e-6)
        || ((aRoot - 3.0).abs() < 1.0e-6);
    assert!(aFoundValidRoot);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
}

#[test]
fn MathNewtonFunctionRootTest__SineRootFinding() {
    // Test finding root of sine function
    let mut aFunc = SineWithDerivative;

    let mut aSolver = NewtonFunctionRoot::new(&mut aFunc, 3.0, 1.0e-10, 1.0e-10); // Guess near PI

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), std::f64::consts::PI, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
    assert_near(aSolver.derivative().unwrap(), -1.0, 1.0e-8);
}

#[test]
fn MathNewtonFunctionRootTest__ExponentialRootFinding() {
    // Test finding root of exponential function
    let mut aFunc = ExponentialWithDerivative;

    let mut aSolver = NewtonFunctionRoot::new(&mut aFunc, 1.0, 1.0e-10, 1.0e-10); // Guess near ln(2)

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0_f64.ln(), 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
    assert_near(aSolver.derivative().unwrap(), 2.0, 1.0e-8);
}
