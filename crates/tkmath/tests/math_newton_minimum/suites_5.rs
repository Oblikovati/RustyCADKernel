#[test]
fn MathNewtonMinimumTest__StatusAccess() {
    // Test status access method
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1e-6, true).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    // Test that we can access the status without exception
    aSolver.status();
}
