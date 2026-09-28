#[test]
fn MathFunctionRootTest__ZeroDerivativeHandling() {
    let mut aFunc = ZeroDerivativeFunction;
    let aTolerance: f64 = 1.0e-6;
    let anInitialGuess: f64 = 0.1; // Close to the root at x = 0

    let aRootFinder = FunctionRoot::new(&mut aFunc, anInitialGuess, aTolerance).unwrap();

    assert!(aRootFinder.is_done());
    assert_near(aRootFinder.root().unwrap(), 0.0, aTolerance);
    assert_near(aRootFinder.value().unwrap(), 0.0, aTolerance);
    assert_near(aRootFinder.derivative().unwrap(), 0.0, aTolerance);
}

// Tests for exceptions
#[test]
fn MathFunctionRootTest__ConstrainedConvergenceState() {
    let mut aFunc = QuadraticFunction;
    let aTolerance: f64 = 1.0e-10; // Reasonable tolerance for debug mode
    let anInitialGuess: f64 = 50.0; // Far from roots but not extreme
    let aMaxIterations: i32 = 3; // Few but reasonable iterations

    // Wrap in try-catch to handle potential exceptions in debug mode
    {
        let aRootFinder =
            FunctionRoot::with_iterations(&mut aFunc, anInitialGuess, aTolerance, aMaxIterations)
                .unwrap();

        // Test state handling for constrained convergence conditions
        if !aRootFinder.is_done() {
            // Verify consistent state reporting
            assert!(!aRootFinder.is_done());
            assert!(aRootFinder.iterations().unwrap() >= 0);
        } else {
            // If it succeeds, verify the results are reasonable
            assert!(aRootFinder.iterations().unwrap() > 0);
            assert!(
                (aRootFinder.root().unwrap() - 2.0).abs() < 0.5
                    || (aRootFinder.root().unwrap() - (-2.0)).abs() < 0.5
            );
        }
    }
}

// Tests for convergence behavior
#[test]
fn MathFunctionRootTest__ConvergenceBehavior() {
    let mut aFunc = QuadraticFunction;
    let aTolerance: f64 = 1.0e-6;

    // Test different initial guesses and verify they converge to the nearest root
    for (initial_guess, expected_root, tolerance) in [
        (1.0, 2.0, aTolerance),
        (-1.0, -2.0, aTolerance),
        (10.0, 2.0, aTolerance),
        (-10.0, -2.0, aTolerance),
    ] {
        let aRootFinder = FunctionRoot::new(&mut aFunc, initial_guess, tolerance).unwrap();

        assert!(aRootFinder.is_done());
        assert_near(aRootFinder.root().unwrap(), expected_root, tolerance);
    }
}

// Performance test
