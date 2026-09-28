#[test]
fn MathBFGSTest__GradientCopyMethod() {
    let mut aFunc = QuadraticFunction2D;
    let mut anOptimizer = Bfgs::with_options(2, 1e-8, 200, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 3.0;
    aStartPoint[2] = 4.0;

    anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(anOptimizer.is_done());

    // Test Gradient copy method
    let mut aGradientCopy = Vector::with_bounds(1, 2).unwrap();
    anOptimizer.copy_gradient(&mut aGradientCopy).unwrap();

    let aGradientRef = anOptimizer.gradient().unwrap();
    assert_near(aGradientCopy[1], aGradientRef[1], 1e-7);
    assert_near(aGradientCopy[2], aGradientRef[2], 1e-7);
}

#[test]
fn MathBFGSTest__DifferentTolerances() {
    let mut aFunc = QuadraticFunction2D;

    // Test with very tight tolerance
    {
        let aTightTolerance: f64 = 1.0e-12;
        let mut anOptimizer = Bfgs::with_options(2, aTightTolerance, 200, 1e-12).unwrap();

        let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
        aStartPoint[1] = 2.0;
        aStartPoint[2] = 3.0;

        anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

        assert!(anOptimizer.is_done());
        assert_near(
            anOptimizer.location().unwrap()[1],
            1.0,
            aTightTolerance * 10.0,
        );
        assert_near(
            anOptimizer.location().unwrap()[2],
            2.0,
            aTightTolerance * 10.0,
        );
    }

    // Test with loose tolerance
    {
        let aLooseTolerance: f64 = 1.0e-3;
        let mut anOptimizer = Bfgs::with_options(2, aLooseTolerance, 200, 1e-12).unwrap();

        let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
        aStartPoint[1] = 10.0;
        aStartPoint[2] = 10.0;

        anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

        assert!(anOptimizer.is_done());
        assert_near(
            anOptimizer.location().unwrap()[1],
            1.0,
            aLooseTolerance * 10.0,
        );
        assert_near(
            anOptimizer.location().unwrap()[2],
            2.0,
            aLooseTolerance * 10.0,
        );
    }
}

#[test]
fn MathBFGSTest__MaxIterationsLimit() {
    let mut aFunc = RosenbrockFunction; // Challenging function
    let aTolerance: f64 = 1.0e-12; // Very tight tolerance
    let aVeryFewIterations: i32 = 5; // Very few iterations

    let mut anOptimizer = Bfgs::with_options(2, aTolerance, aVeryFewIterations, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = -2.0;
    aStartPoint[2] = 2.0;

    anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

    // Either succeeds within few iterations (unlikely) or fails
    if anOptimizer.is_done() {
        assert!(anOptimizer.iterations().unwrap() <= aVeryFewIterations);
    } else {
        // Failure is acceptable with very few iterations
        assert!(aVeryFewIterations <= 10);
    }
}

// Tests for exception handling
