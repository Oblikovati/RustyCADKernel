#[test]
fn MathNewtonMinimumTest__QuadraticBowlOptimization() {
    // Test Newton minimum on simple quadratic bowl function
    let mut aFunc = QuadraticBowlWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0; // Start at (0, 0)
    aStartPoint[2] = 0.0;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-10, 40, 1e-6, true).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    let aLoc = aSolver.location().unwrap();
    assert_near(aLoc[1], 1.0, 1.0e-8);
    assert_near(aLoc[2], 2.0, 1.0e-8);
    assert_near(aSolver.minimum().unwrap(), 0.0, 1.0e-12);
    assert!(aSolver.iterations().unwrap() > 0);
    assert!(aSolver.iterations().unwrap() <= 10);
}

#[test]
fn MathNewtonMinimumTest__RosenbrockOptimization() {
    // Test Newton minimum on challenging Rosenbrock function
    let mut aFunc = RosenbrockWithHessian;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.8; // Start closer to minimum
    aStartPoint[2] = 0.8;

    let mut aSolver = NewtonMinimum::with_options(&aFunc, 1.0e-6, 100, 1e-6, true).unwrap(); // More iterations for challenging function
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    let aLoc = aSolver.location().unwrap();
    assert_near(aLoc[1], 1.0, 2.0e-1);
    assert_near(aLoc[2], 1.0, 2.0e-1);
    assert!(aSolver.minimum().unwrap() < 2.0e-2);
}
