#[test]
fn MathFunctionRootTest__PerformanceComparison() {
    let mut aFunc = QuadraticFunction;
    let aTolerance: f64 = 1.0e-10;
    let anInitialGuess: f64 = 2.1;

    let aRootFinder = FunctionRoot::new(&mut aFunc, anInitialGuess, aTolerance).unwrap();

    assert!(aRootFinder.is_done());
    assert!(aRootFinder.iterations().unwrap() < 50);
    assert_near(aRootFinder.root().unwrap(), 2.0, aTolerance);
}
