#[test]
fn MathNewtonFunctionRootTest__LinearRootFinding() {
    // Test finding root of linear function
    let mut aFunc = LinearWithDerivative;

    let mut aSolver = NewtonFunctionRoot::new(&mut aFunc, 1.0, 1.0e-10, 1.0e-10);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0, 1.0e-10);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
    assert_near(aSolver.derivative().unwrap(), 2.0, 1.0e-10);
    assert!(aSolver.iterations().unwrap() > 0);
    assert!(aSolver.iterations().unwrap() <= 5);
}

#[test]
fn MathNewtonFunctionRootTest__BoundedInterval() {
    // Test Newton method with bounded interval
    let mut aFunc = QuadraticWithDerivative;

    let mut aSolver = NewtonFunctionRoot::bounded(&mut aFunc, 1.5, 1.0e-10, 1.0e-10, 0.0, 3.0, 100); // Bounded in [0, 3]

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0, 1.0e-8);
    assert!(aSolver.root().unwrap() >= 0.0);
    assert!(aSolver.root().unwrap() <= 3.0);
}

#[test]
fn MathNewtonFunctionRootTest__CustomTolerances() {
    // Test with different tolerance values
    let mut aFunc = QuadraticWithDerivative;

    // Loose tolerance
    let mut aSolver1 = NewtonFunctionRoot::new(&mut aFunc, 3.0, 1.0e-3, 1.0e-3);
    assert!(aSolver1.is_done());
    assert_near(aSolver1.root().unwrap(), 2.0, 1.0e-2);

    // Tight tolerance
    let mut aSolver2 = NewtonFunctionRoot::new(&mut aFunc, 3.0, 1.0e-12, 1.0e-12);
    assert!(aSolver2.is_done());
    assert_near(aSolver2.root().unwrap(), 2.0, 1.0e-10);
}
