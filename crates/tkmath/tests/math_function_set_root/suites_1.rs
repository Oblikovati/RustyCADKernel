#[test]
fn math_FunctionSetRoot__SingleVariable() {
    let mut func = SingleVariableSystem;

    let mut tolerance = Vector::with_bounds(1, 1).unwrap();
    tolerance[1] = 1.0e-6;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 1).unwrap();
    startingPoint[1] = 1.5; // Start near positive root

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1].abs(), 2.0, TOLERANCE); // Should find +/-2
}

#[test]
fn math_FunctionSetRoot__OverdeterminedSystem() {
    let mut func = OverdeterminedSystem;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-5;
    tolerance[2] = 1.0e-5;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    // Should find approximate solution x = 2, y = 1
    assert_near(root[1], 2.0, 1.0e-3);
    assert_near(root[2], 1.0, 1.0e-3);
}

#[test]
fn math_FunctionSetRoot__ThreeVariableSystem() {
    let mut func = ThreeVariableSystem;

    let mut tolerance = Vector::with_bounds(1, 3).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;
    tolerance[3] = 1.0e-6;

    let mut solver = FunctionSetRoot::new(&func, &tolerance, 100).unwrap();

    let mut startingPoint = Vector::with_bounds(1, 3).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;
    startingPoint[3] = 0.0;

    solver.perform(&mut func, &startingPoint, false).unwrap();

    assert!(solver.is_done());

    let root = solver.root().unwrap();
    assert_near(root[1], 2.0, TOLERANCE);
    assert_near(root[2], 2.0, TOLERANCE);
    assert_near(root[3], 2.0, TOLERANCE);
}
