#[test]
fn MathFRPRTest__ThreeDimensionalOptimization() {
    // Test FRPR on 3D quadratic function
    let mut aFunc = Quadratic3DFunction;

    let mut aStartPoint = Vector::with_bounds(1, 3).unwrap();
    aStartPoint[1] = 0.0; // Start at (0, 0, 0)
    aStartPoint[2] = 0.0;
    aStartPoint[3] = 0.0;

    let mut aSolver = Frpr::with_options(&aFunc, 1.0e-10, 200, 1e-12).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    let aLoc = aSolver.location().unwrap();
    assert_near(aLoc[1], 1.0, 1.0e-6);
    assert_near(aLoc[2], 2.0, 1.0e-6);
    assert_near(aLoc[3], 3.0, 1.0e-6);
    assert_near(aSolver.minimum().unwrap(), 0.0, 1.0e-10);
}

#[test]
fn MathFRPRTest__CustomTolerance() {
    // Test with different tolerance values
    let mut aFunc = QuadraticBowlFunction;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    // Loose tolerance
    let mut aSolver1 = Frpr::with_options(&aFunc, 1.0e-3, 200, 1e-12).unwrap();
    aSolver1.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver1.is_done());
    assert_near(aSolver1.location().unwrap()[1], 1.0, 1.0e-2);
    assert_near(aSolver1.location().unwrap()[2], 2.0, 1.0e-2);

    // Tight tolerance
    let mut aSolver2 = Frpr::with_options(&aFunc, 1.0e-12, 200, 1e-12).unwrap();
    aSolver2.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver2.is_done());
    assert_near(aSolver2.location().unwrap()[1], 1.0, 1.0e-8);
    assert_near(aSolver2.location().unwrap()[2], 2.0, 1.0e-8);
}

#[test]
fn MathFRPRTest__CustomIterationLimit() {
    // Test with custom iteration limits
    let mut aFunc = RosenbrockFunction; // More challenging function

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    // Few iterations
    let mut aSolver1 = Frpr::with_options(&aFunc, 1.0e-10, 10, 1e-12).unwrap();
    aSolver1.perform(&mut aFunc, &aStartPoint).unwrap();

    if aSolver1.is_done() {
        assert!(aSolver1.iterations().unwrap() <= 10);
    }

    // Many iterations
    let mut aSolver2 = Frpr::with_options(&aFunc, 1.0e-10, 1000, 1e-12).unwrap();
    aSolver2.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver2.is_done());
}
