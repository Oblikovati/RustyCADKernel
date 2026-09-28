#[test]
fn math_FunctionSetRoot__WithBounds() {
    let mut func = LinearSystem2D;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    let mut lowerBound = Vector::with_bounds(1, 2).unwrap();
    lowerBound[1] = 0.0;
    lowerBound[2] = 0.0;

    let mut upperBound = Vector::with_bounds(1, 2).unwrap();
    upperBound[1] = 3.0;
    upperBound[2] = 3.0;

    solver
        .perform_bounded(&mut func, &startingPoint, &lowerBound, &upperBound, false)
        .unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1], 2.0, TOLERANCE);
    assert_near(root[2], 1.0, TOLERANCE);
    assert!(root[1] >= 0.0 - TOLERANCE);
    assert!(root[1] <= 3.0 + TOLERANCE);
    assert!(root[2] >= 0.0 - TOLERANCE);
    assert!(root[2] <= 3.0 + TOLERANCE);
}

#[test]
fn math_FunctionSetRoot__AlternativeConstructor() {
    let mut func = LinearSystem2D;

    let mut solver = FunctionSetRoot::without_tolerance(&func, 100).unwrap(); // No tolerance specified

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    solver.set_tolerance(&tolerance).unwrap(); // Set tolerance separately

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1], 2.0, TOLERANCE);
    assert_near(root[2], 1.0, TOLERANCE);
}

#[test]
fn math_FunctionSetRoot__CustomIterations() {
    let mut func = LinearSystem2D;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 50).unwrap(); // Limited iterations

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());
    assert!(solver.iterations().unwrap() <= 50);
}
