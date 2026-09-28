#[test]
fn math_Uzawa__InitialError() {
    let mut C = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    C[(1, 1)] = 1.0;
    C[(1, 2)] = 1.0;
    C[(2, 1)] = 1.0;
    C[(2, 2)] = -1.0;

    let mut b = Vector::with_bounds(1, 2).unwrap();
    b[1] = 2.0;
    b[2] = 0.0;

    let mut x0 = Vector::with_bounds(1, 2).unwrap();
    x0[1] = 5.0;
    x0[2] = 3.0;

    let solver = Uzawa::new(&C, &b, &x0).unwrap();

    assert!(solver.is_done());

    let initialError = solver.initial_error().unwrap();
    // Initial error = C*x0 - b
    // C*[5,3] - [2,0] = [8,2] - [2,0] = [6,2]
    assert_near(initialError[1], 6.0, TOLERANCE);
    assert_near(initialError[2], 2.0, TOLERANCE);
}

#[test]
fn math_Uzawa__ErrorVector() {
    let mut C = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    C[(1, 1)] = 1.0;
    C[(1, 2)] = 1.0;
    C[(2, 1)] = 1.0;
    C[(2, 2)] = -1.0;

    let mut b = Vector::with_bounds(1, 2).unwrap();
    b[1] = 2.0;
    b[2] = 0.0;

    let mut x0 = Vector::with_bounds(1, 2).unwrap();
    x0[1] = 0.0;
    x0[2] = 0.0;

    let solver = Uzawa::new(&C, &b, &x0).unwrap();

    assert!(solver.is_done());

    let error = solver.error().unwrap();
    let solution = solver.value().unwrap();

    // Error should be solution - starting point
    assert_near(error[1], solution[1] - x0[1], TOLERANCE);
    assert_near(error[2], solution[2] - x0[2], TOLERANCE);
}

#[test]
fn math_Uzawa__DualVariables() {
    let mut C = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    C[(1, 1)] = 1.0;
    C[(1, 2)] = 1.0;
    C[(2, 1)] = 1.0;
    C[(2, 2)] = -1.0;

    let mut b = Vector::with_bounds(1, 2).unwrap();
    b[1] = 2.0;
    b[2] = 0.0;

    let mut x0 = Vector::with_bounds(1, 2).unwrap();
    x0[1] = 0.0;
    x0[2] = 0.0;

    let solver = Uzawa::new(&C, &b, &x0).unwrap();

    assert!(solver.is_done());

    let mut dualVar = Vector::with_bounds(1, 2).unwrap();
    solver.copy_dual(&mut dualVar).unwrap();

    // Dual variables should be valid
    assert!(dualVar[1].is_finite());
    assert!(dualVar[2].is_finite());
}
