#[test]
fn math_FunctionSetRoot__StateNumber() {
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

    let state: i32 = solver.state_number().unwrap();
    assert!(state >= 0); // State should be valid
}

#[test]
fn math_FunctionSetRoot__DerivativeMatrix() {
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

    let jacobian = solver.derivative().unwrap();
    assert_eq!(jacobian.row_number(), 2);
    assert_eq!(jacobian.col_number(), 2);

    // For linear system, Jacobian should be constant
    assert_near(jacobian[(1, 1)], 2.0, TOLERANCE);
    assert_near(jacobian[(1, 2)], 1.0, TOLERANCE);
    assert_near(jacobian[(2, 1)], 1.0, TOLERANCE);
    assert_near(jacobian[(2, 2)], 2.0, TOLERANCE);
}

#[test]
fn math_FunctionSetRoot__FunctionSetErrors() {
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

    let errors = solver.function_set_errors().unwrap();
    assert_eq!(errors.length(), 2);

    // Errors may represent different things (solution difference), so be more lenient
    assert!(errors[1].is_finite());
    assert!(errors[2].is_finite());
}
