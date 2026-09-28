#[test]
fn MathBFGSTest__Paraboloid3DOptimization() {
    let mut aFunc = Paraboloid3D;
    let aTolerance: f64 = 1.0e-8;
    let aMaxIterations: i32 = 100;

    let mut anOptimizer = Bfgs::with_options(3, aTolerance, aMaxIterations, 1e-12).unwrap();

    // Start from point (3, 4, 5), should converge to origin (0, 0, 0)
    let mut aStartPoint = Vector::with_bounds(1, 3).unwrap();
    aStartPoint[1] = 3.0;
    aStartPoint[2] = 4.0;
    aStartPoint[3] = 5.0;

    anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(anOptimizer.is_done());

    let aLocation = anOptimizer.location().unwrap();
    assert_near(aLocation[1], 0.0, 1.0e-6);
    assert_near(aLocation[2], 0.0, 1.0e-6);
    assert_near(aLocation[3], 0.0, 1.0e-6);

    assert_near(anOptimizer.minimum().unwrap(), 0.0, 1.0e-10);
}

#[test]
fn MathBFGSTest__BoundaryConstraints() {
    let mut aFunc = QuadraticFunction2D;
    let aTolerance: f64 = 1.0e-8;
    let aMaxIterations: i32 = 100;

    let mut anOptimizer = Bfgs::with_options(2, aTolerance, aMaxIterations, 1e-12).unwrap();

    // Set boundaries: x in [2, 4], y in [3, 5]
    // True minimum (1,2) is outside these bounds
    // Constrained minimum should be at (2,3)
    let mut aLowerBound = Vector::with_bounds(1, 2).unwrap();
    aLowerBound[1] = 2.0;
    aLowerBound[2] = 3.0;

    let mut anUpperBound = Vector::with_bounds(1, 2).unwrap();
    anUpperBound[1] = 4.0;
    anUpperBound[2] = 5.0;

    anOptimizer
        .set_boundary(&aLowerBound, &anUpperBound)
        .unwrap();

    // Start from point (3, 4)
    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 3.0;
    aStartPoint[2] = 4.0;

    anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(anOptimizer.is_done());

    let aLocation = anOptimizer.location().unwrap();
    assert!(aLocation[1] >= aLowerBound[1]);
    assert!(aLocation[1] <= anUpperBound[1]);
    assert!(aLocation[2] >= aLowerBound[2]);
    assert!(aLocation[2] <= anUpperBound[2]);

    // Should find constrained minimum at (2,3)
    assert_near(aLocation[1], 2.0, 1.0e-6);
    assert_near(aLocation[2], 3.0, 1.0e-6);
}

#[test]
fn MathBFGSTest__LocationCopyMethod() {
    let mut aFunc = QuadraticFunction2D;
    let mut anOptimizer = Bfgs::with_options(2, 1e-8, 200, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 3.0;
    aStartPoint[2] = 4.0;

    anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(anOptimizer.is_done());

    // Test Location copy method
    let mut aLocationCopy = Vector::with_bounds(1, 2).unwrap();
    anOptimizer.copy_location(&mut aLocationCopy).unwrap();

    let aLocationRef = anOptimizer.location().unwrap();
    assert_near(aLocationCopy[1], aLocationRef[1], 1e-7);
    assert_near(aLocationCopy[2], aLocationRef[2], 1e-7);
}
