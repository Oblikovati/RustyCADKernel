#[test]
fn math_FunctionSetRoot__StopOnDivergent() {
    let mut func = NonlinearSystem;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 100.0; // Start far from solution
    startingPoint[2] = 100.0;

    solver.perform(&mut func, &startingPoint, true).unwrap(); // Stop on divergent

    // May or may not converge from this bad starting point
    if !solver.is_done() {
        assert!(solver.is_divergent());
    }
}

#[test]
fn math_FunctionSetRoot__TightTolerances() {
    let mut func = LinearSystem2D;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-10;
    tolerance[2] = 1.0e-10;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1], 2.0, 1.0e-8);
    assert_near(root[2], 1.0, 1.0e-8);
}

#[test]
fn math_FunctionSetRoot__BoundedSolution() {
    let mut func = LinearSystem2D; // Use linear system for more predictable behavior

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    let mut lowerBound = Vector::with_bounds(1, 2).unwrap();
    lowerBound[1] = 0.5;
    lowerBound[2] = 0.5;

    let mut upperBound = Vector::with_bounds(1, 2).unwrap();
    upperBound[1] = 3.0;
    upperBound[2] = 3.0;

    solver
        .perform_bounded(&mut func, &startingPoint, &lowerBound, &upperBound, false)
        .unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    // Check bounds are respected
    assert!(root[1] >= 0.5 - TOLERANCE);
    assert!(root[1] <= 3.0 + TOLERANCE);
    assert!(root[2] >= 0.5 - TOLERANCE);
    assert!(root[2] <= 3.0 + TOLERANCE);

    // Check solution validity for linear system
    assert_near(root[1], 2.0, TOLERANCE);
    assert_near(root[2], 1.0, TOLERANCE);
}
