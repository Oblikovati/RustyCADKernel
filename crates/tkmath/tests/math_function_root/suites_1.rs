#[test]
fn MathFunctionRootTest__QuadraticWithBounds() {
    let mut aFunc = QuadraticFunction;
    let aTolerance: f64 = 1.0e-6;
    let anInitialGuess: f64 = 1.5;
    let aLowerBound: f64 = 1.0;
    let anUpperBound: f64 = 3.0;

    let aRootFinder = FunctionRoot::bounded(
        &mut aFunc,
        anInitialGuess,
        aTolerance,
        aLowerBound,
        anUpperBound,
        100,
    )
    .unwrap();

    assert!(aRootFinder.is_done());
    assert_near(aRootFinder.root().unwrap(), 2.0, aTolerance);
    assert!(aRootFinder.root().unwrap() >= aLowerBound);
    assert!(aRootFinder.root().unwrap() <= anUpperBound);
}

#[test]
fn MathFunctionRootTest__CubicMultipleRoots() {
    let mut aFunc = CubicFunction;
    let aTolerance: f64 = 1.0e-6;

    // Test finding root near x = 1
    {
        let anInitialGuess: f64 = 0.8;
        let aRootFinder = FunctionRoot::new(&mut aFunc, anInitialGuess, aTolerance).unwrap();

        assert!(aRootFinder.is_done());
        assert_near(aRootFinder.root().unwrap(), 1.0, aTolerance);
        assert_near(aRootFinder.value().unwrap(), 0.0, aTolerance);
    }

    // Test finding root near x = 2
    {
        let anInitialGuess: f64 = 1.8;
        let aRootFinder = FunctionRoot::new(&mut aFunc, anInitialGuess, aTolerance).unwrap();

        assert!(aRootFinder.is_done());
        assert_near(aRootFinder.root().unwrap(), 2.0, aTolerance);
        assert_near(aRootFinder.value().unwrap(), 0.0, aTolerance);
    }

    // Test finding root near x = 3
    {
        let anInitialGuess: f64 = 3.2;
        let aRootFinder = FunctionRoot::new(&mut aFunc, anInitialGuess, aTolerance).unwrap();

        assert!(aRootFinder.is_done());
        assert_near(aRootFinder.root().unwrap(), 3.0, aTolerance);
        assert_near(aRootFinder.value().unwrap(), 0.0, aTolerance);
    }
}

#[test]
fn MathFunctionRootTest__TrigonometricFunction() {
    let mut aFunc = SinFunction;
    let aTolerance: f64 = 1.0e-6;
    let anInitialGuess: f64 = 3.5; // Should converge to PI approximately 3.14159

    let aRootFinder = FunctionRoot::new(&mut aFunc, anInitialGuess, aTolerance).unwrap();

    assert!(aRootFinder.is_done());
    assert_near(
        aRootFinder.root().unwrap(),
        std::f64::consts::PI,
        aTolerance,
    );
    assert_near(aRootFinder.value().unwrap(), 0.0, aTolerance);
    assert_near(aRootFinder.derivative().unwrap(), -1.0, aTolerance);
}
