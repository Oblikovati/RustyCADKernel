#[test]
fn math_NewtonFunctionSetRoot__LinearSystemBasic() {
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

    let root = solver.root().unwrap();
    assert_near(root[1], 1.0, TOLERANCE);
    assert_near(root[2], 1.0, TOLERANCE);
}

#[test]
fn math_NewtonFunctionSetRoot__QuadraticSingleVariable() {
    let mut func = QuadraticFunction;

    let mut tolerance = Vector::with_bounds(1, 1).unwrap();
    tolerance[1] = 1.0e-6;

    let mut solver = NewtonFunctionSetRoot::new(&func, &tolerance, 1.0e-6, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 1).unwrap();
    startingPoint[1] = 1.5; // Start near positive root

    solver.perform(&mut func, &startingPoint).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1].abs(), 2.0, TOLERANCE); // Should find +/-2
}
