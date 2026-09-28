#[test]
fn MathNewtonFunctionRootTest__QuadraticRootFinding() {
    // Test finding root of quadratic function
    let mut aFunc = QuadraticWithDerivative;

    let mut aSolver = NewtonFunctionRoot::new(&mut aFunc, 3.0, 1.0e-10, 1.0e-10); // Guess near positive root

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
    assert_near(aSolver.derivative().unwrap(), 4.0, 1.0e-8);
    assert!(aSolver.iterations().unwrap() > 0);
}

#[test]
fn MathNewtonFunctionRootTest__QuadraticNegativeRoot() {
    // Test finding negative root of quadratic function
    let mut aFunc = QuadraticWithDerivative;

    let mut aSolver = NewtonFunctionRoot::new(&mut aFunc, -3.0, 1.0e-10, 1.0e-10); // Guess near negative root

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), -2.0, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
    assert_near(aSolver.derivative().unwrap(), -4.0, 1.0e-8);
}
