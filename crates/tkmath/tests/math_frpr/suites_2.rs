#[test]
fn MathFRPRTest__GradientAccess() {
    // Test gradient vector access
    let mut aFunc = QuadraticBowlFunction;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = Frpr::with_options(&aFunc, 1.0e-10, 200, 1e-12).unwrap();
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
fn MathFRPRTest__LocationAccess() {
    // Test location vector access methods
    let mut aFunc = QuadraticBowlFunction;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = Frpr::with_options(&aFunc, 1.0e-10, 200, 1e-12).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());

    // Test location output method
    let mut aLocOut = Vector::with_bounds(1, 2).unwrap();
    aSolver.copy_location(&mut aLocOut).unwrap();
    assert_near(aLocOut[1], 1.0, 1.0e-6);
    assert_near(aLocOut[2], 2.0, 1.0e-6);
}

#[test]
fn MathFRPRTest__CustomZEPS() {
    // Test with custom ZEPS (machine epsilon) parameter
    let mut aFunc = QuadraticBowlFunction;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = Frpr::with_options(&aFunc, 1.0e-10, 200, 1.0e-15).unwrap();
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap()[1], 1.0, 1.0e-6);
    assert_near(aSolver.location().unwrap()[2], 2.0, 1.0e-6);
}
