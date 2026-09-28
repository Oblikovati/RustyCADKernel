#[test]
fn MathPowellTest__IterationLimit() {
    let mut aFunc = RosenbrockFunction;
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-12, 5, 1e-12).unwrap(); // Very few iterations

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = -2.0;
    aStartPoint[2] = 3.0;

    let mut aDirections = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aDirections[(1, 1)] = 1.0;
    aDirections[(1, 2)] = 0.0;
    aDirections[(2, 1)] = 0.0;
    aDirections[(2, 2)] = 1.0;

    aPowell
        .perform(&mut aFunc, &aStartPoint, &aDirections)
        .unwrap();

    // With only 5 iterations, should not converge for Rosenbrock from this starting point
    assert!(!aPowell.is_done());
}

#[test]
fn MathPowellTest__ToleranceSettings() {
    let mut aFunc = QuadraticFunction;

    // Loose tolerance
    let mut aPowell1 = Powell::with_options(&aFunc, 1.0e-2, 100, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 5.0;
    aStartPoint[2] = 7.0;

    let mut aDirections = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aDirections[(1, 1)] = 1.0;
    aDirections[(1, 2)] = 0.0;
    aDirections[(2, 1)] = 0.0;
    aDirections[(2, 2)] = 1.0;

    aPowell1
        .perform(&mut aFunc, &aStartPoint, &aDirections)
        .unwrap();
    assert!(aPowell1.is_done());

    let aIterationsLoose: i32 = aPowell1.iterations().unwrap();

    // Tight tolerance
    let mut aPowell2 = Powell::with_options(&aFunc, 1.0e-10, 100, 1e-12).unwrap();
    aPowell2
        .perform(&mut aFunc, &aStartPoint, &aDirections)
        .unwrap();
    assert!(aPowell2.is_done());

    let aIterationsTight: i32 = aPowell2.iterations().unwrap();

    // Tighter tolerance usually requires more iterations
    assert!(aIterationsTight >= aIterationsLoose);
}

#[test]
fn MathPowellTest__LocationOutputMethod() {
    let mut aFunc = QuadraticFunction;
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-8, 100, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 4.0;
    aStartPoint[2] = 6.0;

    let mut aDirections = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aDirections[(1, 1)] = 1.0;
    aDirections[(1, 2)] = 0.0;
    aDirections[(2, 1)] = 0.0;
    aDirections[(2, 2)] = 1.0;

    aPowell
        .perform(&mut aFunc, &aStartPoint, &aDirections)
        .unwrap();
    assert!(aPowell.is_done());

    // Test Location() output method
    let mut aLoc = Vector::with_bounds(1, 2).unwrap();
    aPowell.copy_location(&mut aLoc).unwrap();

    assert_near(aLoc[1], 1.0, 1.0e-6);
    assert_near(aLoc[2], 2.0, 1.0e-6);

    // Compare with direct access
    let aLocDirect = aPowell.location().unwrap();
    assert_near(aLoc[1], aLocDirect[1], 1e-7);
    assert_near(aLoc[2], aLocDirect[2], 1e-7);
}
