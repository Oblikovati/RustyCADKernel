#[test]
fn MathNewtonMinimumTest__ThreeDimensionalOptimization() {
    // Test Newton minimum on 3D quadratic function
    let mut aFunc = Quadratic3DWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 3).unwrap();
    aStartPoint[1] = 0.0; // Start at (0, 0, 0)
    aStartPoint[2] = 0.0;
    aStartPoint[3] = 0.0;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1e-6, true).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    let aLoc = aSolver.location().unwrap();
    assert_near(aLoc[1], 1.0, 1.0e-8);
    assert_near(aLoc[2], 2.0, 1.0e-8);
    assert_near(aLoc[3], 3.0, 1.0e-8);
    assert_near(aSolver.minimum().unwrap(), 0.0, 1.0e-12);
}

#[test]
fn MathNewtonMinimumTest__BoundedOptimization() {
    // Test Newton minimum with bounds
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aLeftBound = Vector::with_bounds(1, 2).unwrap();
    aLeftBound[1] = -0.5;
    aLeftBound[2] = -0.5;

    let mut aRightBound = Vector::with_bounds(1, 2).unwrap();
    aRightBound[1] = 1.5;
    aRightBound[2] = 2.5;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1e-6, true).unwrap();
    aSolver.set_boundary(&aLeftBound, &aRightBound).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    let aLoc = aSolver.location().unwrap();
    assert_near(aLoc[1], 1.0, 1.0e-8);
    assert_near(aLoc[2], 2.0, 1.0e-8);
    assert!(aLoc[1] >= -0.5);
    assert!(aLoc[1] <= 1.5);
    assert!(aLoc[2] >= -0.5);
    assert!(aLoc[2] <= 2.5);
}

#[test]
fn MathNewtonMinimumTest__CustomTolerance() {
    // Test with different tolerance values
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    // Loose tolerance
    let mut aSolver1 = NewtonMinimum::with_options(&aFunc, 1.0e-3, 40, 1e-6, true).unwrap();
    aSolver1.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver1.is_done());
    assert_near(aSolver1.location().unwrap()[1], 1.0, 1.0e-2);
    assert_near(aSolver1.location().unwrap()[2], 2.0, 1.0e-2);

    // Tight tolerance
    let mut aSolver2 = NewtonMinimum::with_options(&aFunc, 1.0e-12, 40, 1e-6, true).unwrap();
    aSolver2.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver2.is_done());
    assert_near(aSolver2.location().unwrap()[1], 1.0, 1.0e-10);
    assert_near(aSolver2.location().unwrap()[2], 2.0, 1.0e-10);
}
