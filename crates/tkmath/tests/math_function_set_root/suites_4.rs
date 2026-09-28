#[test]
fn math_FunctionSetRoot__OutputMethods() {
    let mut func = LinearSystem2D;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());

    // Test output methods
    let mut rootCopy = Vector::with_bounds(1, 2).unwrap();
    solver.copy_root(&mut rootCopy).unwrap();
    assert_near(rootCopy[1], 2.0, TOLERANCE);
    assert_near(rootCopy[2], 1.0, TOLERANCE);

    let mut derivativeCopy = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    solver.copy_derivative(&mut derivativeCopy).unwrap();
    assert_near(derivativeCopy[(1, 1)], 2.0, TOLERANCE);
    assert_near(derivativeCopy[(2, 2)], 2.0, TOLERANCE);

    let mut errorsCopy = Vector::with_bounds(1, 2).unwrap();
    solver.copy_function_set_errors(&mut errorsCopy).unwrap();
    assert!(errorsCopy[1].is_finite());
    assert!(errorsCopy[2].is_finite());
}

#[test]
fn math_FunctionSetRoot__IterationCount() {
    let mut func = LinearSystem2D;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());

    let iterations: i32 = solver.iterations().unwrap();
    assert!(iterations > 0);
    assert!(iterations <= 100); // Default max iterations
}

#[test]
fn math_FunctionSetRoot__GoodStartingPoint() {
    let mut func = LinearSystem2D;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 1.99; // Very close to solution
    startingPoint[2] = 1.01;

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());
    assert!(solver.iterations().unwrap() <= 5); // Should converge quickly
}
