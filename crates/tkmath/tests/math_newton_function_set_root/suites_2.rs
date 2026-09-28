#[test]
fn math_NewtonFunctionSetRoot__AlternativeConstructor() {
    let mut func = LinearSystem;

    let mut solver = NewtonFunctionSetRoot::without_step_tolerance(&func, 1.0e-6, 100).unwrap(); // Only function tolerance

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    solver.set_tolerance(&tolerance).unwrap(); // Set x tolerance separately

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
fn math_NewtonFunctionSetRoot__CustomIterations() {
    let mut func = LinearSystem;

    let mut tolerance = Vector::with_bounds(1, 2).unwrap();
    tolerance[1] = 1.0e-6;
    tolerance[2] = 1.0e-6;

    let mut solver = NewtonFunctionSetRoot::new(&func, &tolerance, 1.0e-6, 10).unwrap(); // Limited iterations

    let mut startingPoint = Vector::with_bounds(1, 2).unwrap();
    startingPoint[1] = 0.0;
    startingPoint[2] = 0.0;

    solver.perform(&mut func, &startingPoint).unwrap();

    assert!(solver.is_done());
    assert!(solver.iterations().unwrap() <= 10);
}

#[test]
fn math_NewtonFunctionSetRoot__ConvergenceIterations() {
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
    assert!(solver.iterations().unwrap() > 0);
    assert!(solver.iterations().unwrap() < 20); // Should converge reasonably fast
}
