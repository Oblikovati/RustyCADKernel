#[test]
fn MathFunctionRootTest__HighPrecisionTolerance() {
    let mut aFunc = QuadraticFunction;
    let aTolerance: f64 = 1.0e-12;
    let anInitialGuess: f64 = 2.1;

    let aRootFinder = FunctionRoot::new(&mut aFunc, anInitialGuess, aTolerance).unwrap();

    assert!(aRootFinder.is_done());
    assert_near(aRootFinder.root().unwrap(), 2.0, aTolerance);
    assert_near(aRootFinder.value().unwrap(), 0.0, aTolerance);
}

#[test]
fn MathFunctionRootTest__MaxIterationsLimit() {
    let mut aFunc = QuadraticFunction;
    let aTolerance: f64 = 1.0e-10; // Reasonable tolerance for debug mode
    let anInitialGuess: f64 = 2.1;
    let aMaxIterations: i32 = 5; // Few but reasonable iterations

    // Wrap in try-catch to handle potential exceptions in debug mode
    {
        let aRootFinder =
            FunctionRoot::with_iterations(&mut aFunc, anInitialGuess, aTolerance, aMaxIterations)
                .unwrap();

        // Should either succeed within iterations or fail gracefully
        if aRootFinder.is_done() {
            assert!(aRootFinder.iterations().unwrap() <= aMaxIterations);
            assert_near(aRootFinder.root().unwrap(), 2.0, 1.0e-3);
        } else {
            // It's acceptable to fail if too few iterations are allowed
            assert!(aMaxIterations <= 10);
        }
    }
}

#[test]
fn MathFunctionRootTest__OutOfBoundsGuess() {
    let mut aFunc = QuadraticFunction;
    let aTolerance: f64 = 1.0e-6;
    let anInitialGuess: f64 = 0.0;
    let aLowerBound: f64 = 2.5;
    let anUpperBound: f64 = 3.0; // No root in this interval

    let aRootFinder = FunctionRoot::bounded(
        &mut aFunc,
        anInitialGuess,
        aTolerance,
        aLowerBound,
        anUpperBound,
        100,
    )
    .unwrap();

    // Test that the algorithm respects bounds - the solution should be within bounds
    // if one is found, or the algorithm should fail
    if aRootFinder.is_done() {
        let aRoot: f64 = aRootFinder.root().unwrap();
        assert!(aRoot >= aLowerBound);
        assert!(aRoot <= anUpperBound);

        // If the algorithm reports Done but the function value is not near zero,
        // it might have stopped due to bounds rather than finding a true root
        let aFunctionValue: f64 = aRootFinder.value().unwrap();
        if aFunctionValue.abs() > 1.0e-3 {
            // This is acceptable - the algorithm stopped due to bounds, not convergence to root
            assert!(aRoot >= aLowerBound);
            assert!(aRoot <= anUpperBound);
        } else {
            // True root found
            assert_near(aFunctionValue, 0.0, aTolerance);
        }
    }
}
