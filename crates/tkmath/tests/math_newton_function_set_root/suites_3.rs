#[test]
fn math_NewtonFunctionSetRoot__DerivativeMatrix() {
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
fn math_NewtonFunctionSetRoot__FunctionSetErrors() {
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

    let errors = solver.function_set_errors().unwrap();
    assert_eq!(errors.length(), 2);

    // Errors should be very small at the root
    assert!(errors[1].abs() < 1.0e-5);
    assert!(errors[2].abs() < 1.0e-5);
}

#[test]
fn math_NewtonFunctionSetRoot__OutputMethods() {
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

    // Test output methods
    let mut rootCopy = Vector::with_bounds(1, 2).unwrap();
    solver.copy_root(&mut rootCopy).unwrap();
    assert_near(rootCopy[1], 1.0, TOLERANCE);
    assert_near(rootCopy[2], 1.0, TOLERANCE);

    let mut derivativeCopy = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    solver.copy_derivative(&mut derivativeCopy).unwrap();
    assert_near(derivativeCopy[(1, 1)], 2.0, TOLERANCE);
    assert_near(derivativeCopy[(2, 2)], 2.0, TOLERANCE);

    let mut errorsCopy = Vector::with_bounds(1, 2).unwrap();
    solver.copy_function_set_errors(&mut errorsCopy).unwrap();
    assert!(errorsCopy[1].abs() < 1.0e-5);
    assert!(errorsCopy[2].abs() < 1.0e-5);
}
