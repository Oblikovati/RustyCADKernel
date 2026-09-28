#[test]
fn math_Uzawa__InverseMatrix() {
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

    let invMatrix = solver.inverse_constraints().unwrap();

    // Should be a valid matrix
    assert_eq!(invMatrix.row_number(), 2);
    assert_eq!(invMatrix.col_number(), 2);

    // Elements should be finite
    for i in 1..=2 {
        for j in 1..=2 {
            assert!(invMatrix[(i, j)].is_finite());
        }
    }
}

#[test]
fn math_Uzawa__LargeSystem() {
    // Test with a larger system
    let n: i32 = 4;
    let mut C = Matrix::with_bounds(1, n, 1, n).unwrap();
    for i in 1..=n {
        for j in 1..=n {
            C[(i, j)] = if i == j { 2.0 } else { 0.5 };
        }
    }

    let mut b = Vector::with_bounds(1, n).unwrap();
    for i in 1..=n {
        b[i] = f64::from(i);
    }

    let mut x0 = Vector::with_bounds(1, n).unwrap();
    for i in 1..=n {
        x0[i] = 0.0;
    }

    let solver = Uzawa::new(&C, &b, &x0).unwrap();

    assert!(solver.is_done());

    let solution = solver.value().unwrap();
    for i in 1..=n {
        assert!(solution[i].is_finite());
    }
}

#[test]
fn math_Uzawa__StartingPointNearSolution() {
    // Test where starting point is already near the solution
    let mut C = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    C[(1, 1)] = 1.0;
    C[(1, 2)] = 1.0;
    C[(2, 1)] = 1.0;
    C[(2, 2)] = -1.0;

    let mut b = Vector::with_bounds(1, 2).unwrap();
    b[1] = 2.0;
    b[2] = 0.0;

    let mut x0 = Vector::with_bounds(1, 2).unwrap();
    x0[1] = 1.001; // Very close to solution (1,1)
    x0[2] = 0.999;

    let solver = Uzawa::new(&C, &b, &x0).unwrap();

    assert!(solver.is_done());
    assert!(solver.iterations().unwrap() <= 10); // Should converge quickly

    let solution = solver.value().unwrap();
    assert_near(solution[1], 1.0, TOLERANCE);
    assert_near(solution[2], 1.0, TOLERANCE);
}
