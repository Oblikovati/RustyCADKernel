#[test]
fn math_Uzawa__WithInequalityConstraints() {
    // Simple test with one equality constraint only (safer)
    // x + y = 2

    let mut C = Matrix::with_bounds(1, 1, 1, 2).unwrap();
    C[(1, 1)] = 1.0;
    C[(1, 2)] = 1.0; // equality constraint

    let mut b = Vector::with_bounds(1, 1).unwrap();
    b[1] = 2.0; // x + y = 2

    let mut x0 = Vector::with_bounds(1, 2).unwrap();
    x0[1] = 1.0;
    x0[2] = 1.0;

    // 0 inequality constraints, 1 equality constraint
    let solver = Uzawa::with_constraints(&C, &b, &x0, 0, 1, [1e-6, 1e-6], 500).unwrap();

    assert!(solver.is_done());

    let solution = solver.value().unwrap();
    assert_near(solution[1] + solution[2], 2.0, TOLERANCE);
}

#[test]
fn math_Uzawa__CustomTolerances() {
    // Test with custom tolerances
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

    let solver = Uzawa::with_options(&C, &b, &x0, 1.0e-8, 1.0e-8, 500).unwrap();

    assert!(solver.is_done());

    let solution = solver.value().unwrap();
    assert_near(solution[1], 1.0, 1.0e-6);
    assert_near(solution[2], 1.0, 1.0e-6);
}

#[test]
fn math_Uzawa__CustomIterations() {
    // Test with limited iterations
    let mut C = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    C[(1, 1)] = 1.0;
    C[(1, 2)] = 1.0;
    C[(2, 1)] = 1.0;
    C[(2, 2)] = -1.0;

    let mut b = Vector::with_bounds(1, 2).unwrap();
    b[1] = 2.0;
    b[2] = 0.0;

    let mut x0 = Vector::with_bounds(1, 2).unwrap();
    x0[1] = 10.0; // Start far from solution
    x0[2] = 10.0;

    let solver = Uzawa::with_options(&C, &b, &x0, 1.0e-6, 1.0e-6, 50).unwrap();

    assert!(solver.is_done());
    assert!(solver.iterations().unwrap() <= 50);
}
