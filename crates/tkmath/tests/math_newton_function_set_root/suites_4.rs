#[test]
fn math_NewtonFunctionSetRoot__IterationCount() {
    let mut func = LinearSystem;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = NewtonFunctionSetRoot::new(&func, &tolerance, 1.0e-6, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    solver.perform(&mut func, &startingPoint).unwrap();

    assert!(solver.is_done());

    let iterations: i32 = solver.iterations().unwrap();
    assert!(iterations > 0);
    assert!(iterations <= 100); // Default max iterations
}

#[test]
fn math_NewtonFunctionSetRoot__GoodStartingPoint() {
    let mut func = LinearSystem;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = NewtonFunctionSetRoot::new(&func, &tolerance, 1.0e-6, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.99; // Very close to solution
    startingPoint[2] = 1.01;

    solver.perform(&mut func, &startingPoint).unwrap();

    assert!(solver.is_done());
    assert!(solver.iterations().unwrap() <= 5); // Should converge quickly
}

#[test]
fn math_NewtonFunctionSetRoot__TightTolerances() {
    let mut func = LinearSystem;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-10;
    tolerance[2] = 1.0e-10;

    let mut solver = NewtonFunctionSetRoot::new(&func, &tolerance, 1.0e-10, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    solver.perform(&mut func, &startingPoint).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1], 1.0, 1.0e-8);
    assert_near(root[2], 1.0, 1.0e-8);
}
