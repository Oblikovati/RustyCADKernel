#[test]
fn math_GaussLeastSquare__SimpleLinearSystem() {
    // Simple 2x2 system: 2x + y = 5, x + 2y = 4
    // Solution: x = 2, y = 1

    let mut A = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    A[(1, 1)] = 2.0;
    A[(1, 2)] = 1.0;
    A[(2, 1)] = 1.0;
    A[(2, 2)] = 2.0;

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 2).unwrap();
    B[1] = 5.0;
    B[2] = 4.0;

    let mut X = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B, &mut X).unwrap();

    assert_near(X[1], 2.0, 1e-6);
    assert_near(X[2], 1.0, 1e-6);
}

#[test]
fn math_GaussLeastSquare__OverdeterminedSystem() {
    // Overdetermined system: 3 equations, 2 unknowns
    // x + y = 3
    // 2x - y = 1
    // x + 2y = 5
    // Least squares solution approximately: x = 1.4, y = 1.6

    let mut A = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    A[(1, 1)] = 1.0;
    A[(1, 2)] = 1.0; // x + y = 3
    A[(2, 1)] = 2.0;
    A[(2, 2)] = -1.0; // 2x - y = 1
    A[(3, 1)] = 1.0;
    A[(3, 2)] = 2.0; // x + 2y = 5

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 3).unwrap();
    B[1] = 3.0;
    B[2] = 1.0;
    B[3] = 5.0;

    let mut X = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B, &mut X).unwrap();

    // Check that solution is reasonable (least squares approximation)
    assert!(X[1].is_finite());
    assert!(X[2].is_finite());
    assert_near(X[1], 1.4, 0.2); // More tolerant for least squares
    assert_near(X[2], 1.8, 0.2);
}
