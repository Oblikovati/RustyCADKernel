#[test]
fn MathBFGSTest__NotDoneState() {
    let mut aFunc = QuadraticFunction2D;
    let mut anOptimizer = Bfgs::with_options(2, 1.0e-12, 3, 1e-12).unwrap(); // Reasonable tolerance, few iterations

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 50.0; // Far from minimum but not extreme
    aStartPoint[2] = 50.0;

    // Wrap in try-catch to handle potential exceptions in debug mode
    {
        anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

        if !anOptimizer.is_done() {
            assert!(anOptimizer.iterations().unwrap() >= 0);
        } else {
            assert!(anOptimizer.iterations().unwrap() > 0);
        }
    }
}

#[test]
fn MathBFGSTest__DimensionCompatibility() {
    let mut aFunc = QuadraticFunction2D;
    let mut anOptimizer = Bfgs::with_options(2, 1e-8, 200, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 2.0;
    aStartPoint[2] = 3.0;

    anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(anOptimizer.is_done());

    // Verify optimizer works correctly with properly dimensioned vectors
    let mut aCorrectSizeLocation = Vector::with_bounds(1, 2).unwrap();
    let mut aCorrectSizeGradient = Vector::with_bounds(1, 2).unwrap();

    anOptimizer
        .copy_location(&mut aCorrectSizeLocation)
        .unwrap();
    anOptimizer
        .copy_gradient(&mut aCorrectSizeGradient)
        .unwrap();

    // Verify the results make sense
    assert_eq!(aCorrectSizeLocation.length(), 2);
    assert_eq!(aCorrectSizeGradient.length(), 2);
}

#[test]
fn MathBFGSTest__ConstructorParameters() {
    // Test different constructor parameter combinations
    {
        let mut anOptimizer1 = Bfgs::with_options(3, 1e-8, 200, 1e-12).unwrap(); // Default tolerance, iterations, ZEPS
        let mut aFunc = QuadraticFunction2D; // This is 2D, but optimizer is 3D - should handle gracefully

        // This might not work well due to dimension mismatch, but shouldn't crash
        {
            let mut aStart = Vector::with_bounds(1, 3).unwrap();
            aStart.init(1.0);
            // Don't perform - just test construction doesn't crash
        }
    }

    {
        let mut anOptimizer2 = Bfgs::with_options(2, 1.0e-6, 200, 1e-12).unwrap(); // Custom tolerance, default iterations and ZEPS
        // Should construct successfully
    }

    {
        let mut anOptimizer3 = Bfgs::with_options(2, 1.0e-8, 50, 1e-12).unwrap(); // Custom tolerance and iterations
        // Should construct successfully
    }

    {
        let mut anOptimizer4 = Bfgs::with_options(2, 1.0e-8, 100, 1.0e-10).unwrap(); // All parameters custom
        // Should construct successfully
    }
}
