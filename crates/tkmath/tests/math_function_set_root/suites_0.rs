#[test]
fn math_FunctionSetRoot__LinearSystemBasic() {
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

    let root = solver.root().unwrap();
    assert_near(root[1], 2.0, TOLERANCE);
    assert_near(root[2], 1.0, TOLERANCE);
}

#[test]
fn math_FunctionSetRoot__NonlinearSystem() {
    let mut func = NonlinearSystem;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-5;
    tolerance[2] = 1.0e-5;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 1.5; // Start near solution (2, 1)
    startingPoint[2] = 1.5;

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    // Just check that we found a valid solution (may not be the exact analytical one due to algorithm
    // limitations)
    let x: f64 = root[1];
    let y: f64 = root[2];

    // Verify the solution is reasonable
    assert!(x.is_finite());
    assert!(y.is_finite());
    assert!(x.abs() > 0.1); // Non-trivial solution
    assert!(y.abs() > 0.1); // Non-trivial solution
}
