#[test]
fn MathFRPRTest__QuadraticBowlOptimization() {
    // Test FRPR on simple quadratic bowl function
    let mut aFunc = QuadraticBowlFunction;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0; // Start at (0, 0)
    aStartPoint[2] = 0.0;

    let mut aSolver = Frpr::with_options(&aFunc, 1.0e-10, 200, 1e-12).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    let aLoc = aSolver.location().unwrap();
    assert_near(aLoc[1], 1.0, 1.0e-6);
    assert_near(aLoc[2], 2.0, 1.0e-6);
    assert_near(aSolver.minimum().unwrap(), 0.0, 1.0e-10);
    assert!(aSolver.iterations().unwrap() > 0);
}

#[test]
fn MathFRPRTest__RosenbrockOptimization() {
    // Test FRPR on the challenging Rosenbrock function
    let mut aFunc = RosenbrockFunction;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0; // Start at (0, 0)
    aStartPoint[2] = 0.0;

    let mut aSolver = Frpr::with_options(&aFunc, 1.0e-8, 500, 1e-12).unwrap(); // More iterations for challenging function
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    let aLoc = aSolver.location().unwrap();
    assert_near(aLoc[1], 1.0, 1.0e-3);
    assert_near(aLoc[2], 1.0, 1.0e-3);
    assert!(aSolver.minimum().unwrap() < 1.0e-4);
}
