#[test]
fn MathPowellTest__RosenbrockFunction() {
    let mut aFunc = RosenbrockFunction;
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-6, 1000, 1e-12).unwrap(); // More iterations for challenging function

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = -1.0;
    aStartPoint[2] = 1.0; // Classic starting point

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

    // Rosenbrock is challenging - allow larger tolerance
    assert_near(aLocation[1], 1.0, 1.0e-3);
    assert_near(aLocation[2], 1.0, 1.0e-3);
    assert_near(aPowell.minimum().unwrap(), 0.0, 1.0e-5);
}

#[test]
fn MathPowellTest__HigherDimensionalOptimization() {
    let mut aFunc = MultiDimensionalQuadratic(4); // 4D optimization
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-8, 200, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 4).unwrap();
    aStartPoint[1] = 0.0;
    aStartPoint[2] = 0.0;
    aStartPoint[3] = 0.0;
    aStartPoint[4] = 0.0;

    // Identity matrix for initial directions
    let mut aDirections = Matrix::with_bounds(1, 4, 1, 4).unwrap();
    for i in 1..=4 {
        for j in 1..=4 {
            aDirections[(i, j)] = if i == j { 1.0 } else { 0.0 };
        }
    }

    aPowell
        .perform(&mut aFunc, &aStartPoint, &aDirections)
        .unwrap();

    assert!(aPowell.is_done());

    let aLocation = aPowell.location().unwrap();

    // Expected optimum: (1, 2, 3, 4)
    assert_near(aLocation[1], 1.0, 1.0e-6);
    assert_near(aLocation[2], 2.0, 1.0e-6);
    assert_near(aLocation[3], 3.0, 1.0e-6);
    assert_near(aLocation[4], 4.0, 1.0e-6);
    assert_near(aPowell.minimum().unwrap(), 0.0, 1.0e-10);
}

#[test]
fn MathPowellTest__DifferentStartingDirections() {
    let mut aFunc = QuadraticFunction;
    let mut aPowell = Powell::with_options(&aFunc, 1.0e-8, 100, 1e-12).unwrap();

    let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
    aStartPoint[1] = 3.0;
    aStartPoint[2] = 5.0;

    // Non-orthogonal starting directions
    let mut aDirections = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aDirections[(1, 1)] = 1.0;
    aDirections[(1, 2)] = 1.0; // [1, 1]
    aDirections[(2, 1)] = 1.0;
    aDirections[(2, 2)] = -1.0; // [1, -1]

    aPowell
        .perform(&mut aFunc, &aStartPoint, &aDirections)
        .unwrap();

    assert!(aPowell.is_done());

    let aLocation = aPowell.location().unwrap();
    assert_near(aLocation[1], 1.0, 1.0e-6);
    assert_near(aLocation[2], 2.0, 1.0e-6);
}
