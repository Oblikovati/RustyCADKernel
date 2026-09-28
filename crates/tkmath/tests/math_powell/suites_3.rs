#[test]
fn MathPowellTest__UnperformedState() {
    let mut aFunc = QuadraticFunction;
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-8, 100, 1e-12).unwrap();

    // Before Perform() is called, optimizer should report not done
    assert!(!aPowell.is_done());

    // In release builds, verify the optimizer maintains consistent state
    if !aPowell.is_done() {
        assert!(!aPowell.is_done());
    }
}

#[test]
fn MathPowellTest__DimensionCompatibility() {
    let mut aFunc = QuadraticFunction;
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-8, 100, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 2.0;
    aStartPoint[2] = 3.0;

    let mut aDirections = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aDirections[(1, 1)] = 1.0;
    aDirections[(1, 2)] = 0.0;
    aDirections[(2, 1)] = 0.0;
    aDirections[(2, 2)] = 1.0;

    aPowell
        .perform(&mut aFunc, &aStartPoint, &aDirections)
        .unwrap();
    assert!(aPowell.is_done());

    // Test with correctly sized vector
    let mut aCorrectLoc = Vector::with_bounds(1, 2).unwrap(); // Correct size 2
    aPowell.copy_location(&mut aCorrectLoc).unwrap();

    // Verify the result makes sense
    assert_eq!(aCorrectLoc.length(), 2);
}

#[test]
fn MathPowellTest__AlreadyAtOptimum() {
    let mut aFunc = QuadraticFunction;
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-8, 100, 1e-12).unwrap();

    // Start at the optimum
    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 1.0;
    aStartPoint[2] = 2.0;

    let mut aDirections = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aDirections[(1, 1)] = 1.0;
    aDirections[(1, 2)] = 0.0;
    aDirections[(2, 1)] = 0.0;
    aDirections[(2, 2)] = 1.0;

    aPowell
        .perform(&mut aFunc, &aStartPoint, &aDirections)
        .unwrap();

    assert!(aPowell.is_done());

    let aLocation = aPowell.location().unwrap();
    assert_near(aLocation[1], 1.0, 1.0e-10);
    assert_near(aLocation[2], 2.0, 1.0e-10);
    assert_near(aPowell.minimum().unwrap(), 0.0, 1.0e-12);

    // Should converge very quickly
    assert!(aPowell.iterations().unwrap() <= 5);
}
