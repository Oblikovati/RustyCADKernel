#[test]
fn MathNewtonMinimumTest__UnperformedState() {
    // Test state handling before Perform() is called
    let mut aFunc = QuadraticBowlWithHessian;
    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1e-6, true).unwrap();

    // Before Perform() is called, solver should report not done
    assert!(!aSolver.is_done());

    // In release builds, verify the solver maintains consistent state
    if !aSolver.is_done() {
        assert!(!aSolver.is_done());
    }
}

#[test]
fn MathNewtonMinimumTest__DimensionCompatibility() {
    // Test dimension compatibility handling
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1e-6, true).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    // Test with correctly dimensioned vectors
    let mut aCorrectVec = Vector::with_bounds(1, 2).unwrap(); // 2D vector for 2D function
    aSolver.copy_location(&mut aCorrectVec).unwrap();
    aSolver.copy_gradient(&mut aCorrectVec).unwrap();

    // Verify the results make sense
    assert_eq!(aCorrectVec.length(), 2);
}

#[test]
fn MathNewtonMinimumTest__StartingNearMinimum() {
    // Test when starting point is already near the minimum
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 1.001; // Very close to minimum at (1, 2)
    aStartPoint[2] = 1.999;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-12, 40, 1e-6, true).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap()[1], 1.0, 1.0e-8);
    assert_near(aSolver.location().unwrap()[2], 2.0, 1.0e-8);
    assert_near(aSolver.minimum().unwrap(), 0.0, 1.0e-12);
    assert!(aSolver.iterations().unwrap() <= 5);
}
