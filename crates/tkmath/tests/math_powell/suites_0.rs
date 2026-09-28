#[test]
fn MathPowellTest__SimpleQuadraticFunction() {
    let mut aFunc = QuadraticFunction;
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-8, 100, 1e-12).unwrap();

    // Starting point away from minimum
    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 5.0;
    aStartPoint[2] = 7.0;

    // Initial search directions (identity matrix)
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
    let aMinimum: f64 = aPowell.minimum().unwrap();

    assert_near(aLocation[1], 1.0, 1.0e-6);
    assert_near(aLocation[2], 2.0, 1.0e-6);
    assert_near(aMinimum, 0.0, 1.0e-10);
}

#[test]
fn MathPowellTest__Simple1DOptimization() {
    let mut aFunc = Simple1DFunction;
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-10, 50, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 1).unwrap();
    aStartPoint[1] = 10.0; // Start far from optimum

    let mut aDirections = Matrix::with_bounds(1, 1, 1, 1).unwrap();
    aDirections[(1, 1)] = 1.0; // Single direction

    aPowell
        .perform(&mut aFunc, &aStartPoint, &aDirections)
        .unwrap();

    assert!(aPowell.is_done());

    let aLocation = aPowell.location().unwrap();
    assert_near(aLocation[1], 3.0, 1.0e-8);
    assert_near(aPowell.minimum().unwrap(), 0.0, 1.0e-12);
}
