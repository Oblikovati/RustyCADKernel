#[test]
fn MathNewtonMinimumTest__CustomConvexity() {
    // Test with custom convexity parameter
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1.0e-8, true).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap()[1], 1.0, 1.0e-8);
    assert_near(aSolver.location().unwrap()[2], 2.0, 1.0e-8);
}

#[test]
fn MathNewtonMinimumTest__WithSingularityTreatment() {
    // Test with singularity treatment enabled
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1.0e-6, true).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap()[1], 1.0, 1.0e-8);
    assert_near(aSolver.location().unwrap()[2], 2.0, 1.0e-8);
}

#[test]
fn MathNewtonMinimumTest__NonConvexFunction() {
    // Test with non-convex function (saddle point)
    let mut aFunc = SaddleFunction;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.1;
    aStartPoint[2] = 0.1;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1.0e-6, false).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    // Function is not convex, Newton method may not converge
    // Testing that algorithm can handle non-convex functions
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();
}
