#[test]
fn math_Uzawa__ConsistentSystem() {
    // Test with consistent system
    let mut C = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    C[(1, 1)] = 1.0;
    C[(1, 2)] = 2.0;
    C[(2, 1)] = 2.0;
    C[(2, 2)] = 1.0;

    let mut b = Vector::with_bounds(1, 2).unwrap();
    b[1] = 5.0;
    b[2] = 7.0;

    let mut x0 = Vector::with_bounds(1, 2).unwrap();
    x0[1] = 0.0;
    x0[2] = 0.0;

    let solver = Uzawa::new(&C, &b, &x0).unwrap();

    assert!(solver.is_done());

    // Verify solution satisfies the equations
    let solution = solver.value().unwrap();
    let eq1: f64 = solution[1] + 2.0 * solution[2];
    let eq2: f64 = 2.0 * solution[1] + solution[2];
    assert_near(eq1, 5.0, TOLERANCE);
    assert_near(eq2, 7.0, TOLERANCE);
}

#[test]
fn math_Uzawa__SingleVariable() {
    // Test with single variable system: 2x = 4
    let mut C = Matrix::with_bounds(1, 1, 1, 1).unwrap();
    C[(1, 1)] = 2.0;

    let mut b = Vector::with_bounds(1, 1).unwrap();
    b[1] = 4.0;

    let mut x0 = Vector::with_bounds(1, 1).unwrap();
    x0[1] = 0.0;

    let solver = Uzawa::new(&C, &b, &x0).unwrap();

    assert!(solver.is_done());

    let solution = solver.value().unwrap();
    assert_near(solution[1], 2.0, TOLERANCE);
}

#[test]
fn math_Uzawa__IterationCount() {
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
    assert!(solver.iterations().unwrap() > 0);
    assert!(solver.iterations().unwrap() <= 500); // Default max iterations
}
