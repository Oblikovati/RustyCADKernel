#[test]
fn math_Uzawa__SimpleEqualityConstraints() {
    // Simple 2x2 system with equality constraints:
    // 2x + y = 5
    // x + 2y = 4
    // Expected solution: x = 2, y = 1

    let mut C = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    C[(1, 1)] = 2.0;
    C[(1, 2)] = 1.0;
    C[(2, 1)] = 1.0;
    C[(2, 2)] = 2.0;

    let mut b = Vector::with_bounds(1, 2).unwrap();
    b[1] = 5.0;
    b[2] = 4.0;

    let mut x0 = Vector::with_bounds(1, 2).unwrap();
    x0[1] = 0.0;
    x0[2] = 0.0;

    let solver = Uzawa::new(&C, &b, &x0).unwrap();

    assert!(solver.is_done());

    let solution = solver.value().unwrap();
    assert_near(solution[1], 2.0, TOLERANCE);
    assert_near(solution[2], 1.0, TOLERANCE);
}

#[test]
fn math_Uzawa__OverdeterminedSystem() {
    // Simple overdetermined but consistent system
    // x + y = 3
    // x - y = 1
    // Expected solution: x = 2, y = 1

    let mut C = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    C[(1, 1)] = 1.0;
    C[(1, 2)] = 1.0;
    C[(2, 1)] = 1.0;
    C[(2, 2)] = -1.0;

    let mut b = Vector::with_bounds(1, 2).unwrap();
    b[1] = 3.0;
    b[2] = 1.0;

    let mut x0 = Vector::with_bounds(1, 2).unwrap();
    x0[1] = 0.0;
    x0[2] = 0.0;

    let solver = Uzawa::new(&C, &b, &x0).unwrap();

    assert!(solver.is_done());

    let solution = solver.value().unwrap();
    assert_near(solution[1], 2.0, TOLERANCE);
    assert_near(solution[2], 1.0, TOLERANCE);
}
