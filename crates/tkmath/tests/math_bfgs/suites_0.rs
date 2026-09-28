#[test]
fn MathBFGSTest__QuadraticFunction2DOptimization() {
    let mut aFunc = QuadraticFunction2D;
    let aTolerance: f64 = 1.0e-8;
    let aMaxIterations: i32 = 100;

    let mut anOptimizer = Bfgs::with_options(2, aTolerance, aMaxIterations, 1e-12).unwrap();

    // Start from point (5, 7), should converge to (1, 2)
    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 5.0;
    aStartPoint[2] = 7.0;

    anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(anOptimizer.is_done());

    let aLocation = anOptimizer.location().unwrap();
    assert_near(aLocation[1], 1.0, 1.0e-6);
    assert_near(aLocation[2], 2.0, 1.0e-6);

    assert_near(anOptimizer.minimum().unwrap(), 0.0, 1.0e-10);

    let aGradient = anOptimizer.gradient().unwrap();
    assert_near(aGradient[1], 0.0, 1.0e-6);
    assert_near(aGradient[2], 0.0, 1.0e-6);

    assert!(anOptimizer.iterations().unwrap() > 0);
    assert!(anOptimizer.iterations().unwrap() <= aMaxIterations);
}

#[test]
fn MathBFGSTest__RosenbrockFunctionOptimization() {
    let mut aFunc = RosenbrockFunction;
    let aTolerance: f64 = 1.0e-6;
    let aMaxIterations: i32 = 1000; // Rosenbrock can be challenging

    let mut anOptimizer = Bfgs::with_options(2, aTolerance, aMaxIterations, 1e-12).unwrap();

    // Start from point (-1, 1), should converge to (1, 1)
    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = -1.0;
    aStartPoint[2] = 1.0;

    anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(anOptimizer.is_done());

    let aLocation = anOptimizer.location().unwrap();
    assert_near(aLocation[1], 1.0, 1.0e-4);
    assert_near(aLocation[2], 1.0, 1.0e-4);

    assert_near(anOptimizer.minimum().unwrap(), 0.0, 1.0e-8);
}
