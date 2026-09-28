#[test]
fn MathFunctionRootTest__QuadraticPositiveRoot() {
    let mut aFunc = QuadraticFunction;
    let aTolerance: f64 = 1.0e-6;
    let anInitialGuess: f64 = 3.0; // Should converge to +2

    let aRootFinder = FunctionRoot::new(&mut aFunc, anInitialGuess, aTolerance).unwrap();

    assert!(aRootFinder.is_done());
    assert_near(aRootFinder.root().unwrap(), 2.0, aTolerance);
    assert_near(aRootFinder.value().unwrap(), 0.0, aTolerance);
    assert_near(aRootFinder.derivative().unwrap(), 4.0, aTolerance);
    assert!(aRootFinder.iterations().unwrap() > 0);
}

#[test]
fn MathFunctionRootTest__QuadraticNegativeRoot() {
    let mut aFunc = QuadraticFunction;
    let aTolerance: f64 = 1.0e-6;
    let anInitialGuess: f64 = -3.0; // Should converge to -2

    let aRootFinder = FunctionRoot::new(&mut aFunc, anInitialGuess, aTolerance).unwrap();

    assert!(aRootFinder.is_done());
    assert_near(aRootFinder.root().unwrap(), -2.0, aTolerance);
    assert_near(aRootFinder.value().unwrap(), 0.0, aTolerance);
    assert_near(aRootFinder.derivative().unwrap(), -4.0, aTolerance);
}
