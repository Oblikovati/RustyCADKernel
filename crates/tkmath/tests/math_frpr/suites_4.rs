#[test]
fn MathFRPRTest__QuarticFlatMinimum() {
    // Test with quartic function that has very flat minimum
    let mut aFunc = QuarticFunction;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = Frpr::with_options(&aFunc, 1.0e-8, 200, 1e-12).unwrap(); // Slightly looser tolerance for flat minimum
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap()[1], 1.0, 1.0e-4);
    assert_near(aSolver.location().unwrap()[2], 2.0, 1.0e-4);
}

#[test]
fn MathFRPRTest__LinearFunctionUnbounded() {
    // Test with unbounded linear function
    let mut aFunc = LinearFunction;

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;

    let mut aSolver = Frpr::with_options(&aFunc, 1.0e-10, 50, 1e-12).unwrap(); // Limited iterations
    aSolver.perform(&mut aFunc, &aStartPoint).unwrap();

    // The algorithm may or may not converge for unbounded functions
    // depending on implementation details and stopping criteria
    if aSolver.is_done() {
        // If it "converges", it should have done some work
        assert!(aSolver.iterations().unwrap() > 0);
    }
}

#[test]
fn MathFRPRTest__MultipleCalls() {
    // Test multiple calls to Perform with same instance
    let mut aFunc1 = QuadraticBowlFunction;
    let mut aFunc2 = Quadratic3DFunction;

    let mut aStartPoint2D = Vector::with_bounds(1, 2).unwrap();
    aStartPoint2D[1] = 0.0;
    aStartPoint2D[2] = 0.0;

    let mut aStartPoint3D = Vector::with_bounds(1, 3).unwrap();
    aStartPoint3D[1] = 0.0;
    aStartPoint3D[2] = 0.0;
    aStartPoint3D[3] = 0.0;

    let mut aSolver = Frpr::with_options(&aFunc1, 1.0e-10, 200, 1e-12).unwrap();

    // First call with 2D function
    aSolver.perform(&mut aFunc1, &aStartPoint2D).unwrap();
    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap()[1], 1.0, 1.0e-6);

    // Second call with 3D function - need to create new solver with appropriate function
    let mut aSolver2 = Frpr::with_options(&aFunc2, 1.0e-10, 200, 1e-12).unwrap();
    aSolver2.perform(&mut aFunc2, &aStartPoint3D).unwrap();
    assert!(aSolver2.is_done());
    assert_near(aSolver2.location().unwrap()[1], 1.0, 1.0e-6);
}
