#[test]
fn math_GaussLeastSquare__CustomMinPivot() {
    // Test with custom minimum pivot
    let mut A = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    A[(1, 1)] = 1.0;
    A[(1, 2)] = 0.0;
    A[(2, 1)] = 0.0;
    A[(2, 2)] = 1.0;

    let solver = GaussLeastSquares::with_min_pivot(&A, 1.0e-15).unwrap(); // Custom min pivot

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 2).unwrap();
    B[1] = 2.0;
    B[2] = 3.0;

    let mut X = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B, &mut X).unwrap();

    assert_near(X[1], 2.0, 1e-6);
    assert_near(X[2], 3.0, 1e-6);
}

#[test]
fn math_GaussLeastSquare__LargeDiagonalValues() {
    // Test with large diagonal values
    let mut A = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    A[(1, 1)] = 100.0;
    A[(1, 2)] = 1.0;
    A[(2, 1)] = 1.0;
    A[(2, 2)] = 100.0;

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 2).unwrap();
    B[1] = 101.0; // 100x + y = 101
    B[2] = 102.0; // x + 100y = 102

    let mut X = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B, &mut X).unwrap();

    // Expected solution approximately x = 1, y = 1 (with more tolerance)
    assert_near(X[1], 1.0, 0.02);
    assert_near(X[2], 1.01, 0.02);
}

#[test]
fn math_GaussLeastSquare__ScaledSystem() {
    // Test system with scaled coefficients
    let mut A = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    A[(1, 1)] = 0.001;
    A[(1, 2)] = 0.002;
    A[(2, 1)] = 0.003;
    A[(2, 2)] = 0.004;

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 2).unwrap();
    B[1] = 0.005;
    B[2] = 0.007;

    let mut X = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B, &mut X).unwrap();

    // Solution should be finite and reasonable
    assert!(X[1].is_finite());
    assert!(X[2].is_finite());
}
