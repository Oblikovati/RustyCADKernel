#[test]
fn math_NewtonFunctionSetRoot__CircleLineIntersection() {
    let mut func = CircleLineSystem;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = NewtonFunctionSetRoot::new(&func, &tolerance, 1.0e-6, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.7; // Start near positive solution
    startingPoint[2] = 0.7;

    solver.perform(&mut func, &startingPoint).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1].abs(), std::f64::consts::FRAC_1_SQRT_2, 1.0e-5);
    assert_near(root[2].abs(), std::f64::consts::FRAC_1_SQRT_2, 1.0e-5);
    assert_near(root[1], root[2], TOLERANCE); // x = y constraint
}

#[test]
fn math_NewtonFunctionSetRoot__ThreeVariableSystem() {
    let mut func = ThreeVariableSystem;

    let mut tolerance = Vector::with_bounds(1, 3).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;
    tolerance[3] = 1.0e-6;

    let mut solver = NewtonFunctionSetRoot::new(&func, &tolerance, 1.0e-6, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 3).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;
    startingPoint[3] = 0.0;

    solver.perform(&mut func, &startingPoint).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1], 2.0, TOLERANCE);
    assert_near(root[2], 2.0, TOLERANCE);
    assert_near(root[3], 2.0, TOLERANCE);
}

#[test]
fn math_NewtonFunctionSetRoot__WithBounds() {
    let mut func = LinearSystem;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = NewtonFunctionSetRoot::new(&func, &tolerance, 1.0e-6, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    let mut lowerBound = Vector::with_bounds(1, 2).unwrap();
    lowerBound[1] = 0.0;
    lowerBound[2] = 0.0;

    let mut upperBound = Vector::with_bounds(1, 2).unwrap();
    upperBound[1] = 2.0;
    upperBound[2] = 2.0;

    solver
        .perform_bounded(&mut func, &startingPoint, &lowerBound, &upperBound)
        .unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1], 1.0, TOLERANCE);
    assert_near(root[2], 1.0, TOLERANCE);
    assert!(root[1] >= 0.0 - TOLERANCE);
    assert!(root[1] <= 2.0 + TOLERANCE);
    assert!(root[2] >= 0.0 - TOLERANCE);
    assert!(root[2] <= 2.0 + TOLERANCE);
}
