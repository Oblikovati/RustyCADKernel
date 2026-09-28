#[test]
fn MathNewtonMinimumTest__CustomIterationLimit() {
    // Test with custom iteration limits
    let mut aFunc = RosenbrockWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.8;
    aStartPoint[2] = 0.8;

    // Few iterations
    let mut aSolver1 = NewtonMinimum::with_options(&aFunc, 1.0e-6, 5, 1e-6, true).unwrap();
    aSolver1.perform(&mut aFunc, &aStartPoint).unwrap();

    if aSolver1.is_done() {
        assert!(aSolver1.iterations().unwrap() <= 5);
    }

    // Many iterations
    let mut aSolver2 = NewtonMinimum::with_options(&aFunc, 1.0e-8, 200, 1e-6, true).unwrap();
    aSolver2.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver2.is_done());
}

#[test]
fn MathNewtonMinimumTest__GradientAccess() {
    // Test gradient vector access
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1e-6, true).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    let aGrad = aSolver.gradient().unwrap();
    assert_near(aGrad[1], 0.0, 1.0e-8);
    assert_near(aGrad[2], 0.0, 1.0e-8);

    // Test gradient output method
    let mut aGradOut = Vector::with_bounds(1, 2).unwrap();
    aSolver.copy_gradient(&mut aGradOut).unwrap();
    assert_near(aGradOut[1], 0.0, 1.0e-8);
    assert_near(aGradOut[2], 0.0, 1.0e-8);
}

#[test]
fn MathNewtonMinimumTest__LocationAccess() {
    // Test location vector access methods
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1e-6, true).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    // Test location output method
    let mut aLocOut = Vector::with_bounds(1, 2).unwrap();
    aSolver.copy_location(&mut aLocOut).unwrap();
    assert_near(aLocOut[1], 1.0, 1.0e-8);
    assert_near(aLocOut[2], 2.0, 1.0e-8);
}
