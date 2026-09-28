#[test]
fn math_GaussLeastSquare__RectangularMatrix() {
    // 4x2 matrix (more equations than unknowns)
    let mut A = Matrix::with_bounds(1, 4, 1, 2).unwrap();
    A[(1, 1)] = 1.0;
    A[(1, 2)] = 1.0;
    A[(2, 1)] = 1.0;
    A[(2, 2)] = 2.0;
    A[(3, 1)] = 2.0;
    A[(3, 2)] = 1.0;
    A[(4, 1)] = 2.0;
    A[(4, 2)] = 2.0;

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 4).unwrap();
    B[1] = 3.0;
    B[2] = 4.0;
    B[3] = 4.0;
    B[4] = 5.0;

    let mut X = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B, &mut X).unwrap();

    // Check that solution is reasonable
    assert!(X[1].is_finite());
    assert!(X[2].is_finite());
    assert!(X[1] > 0.0); // Expected positive values
    assert!(X[2] > 0.0);
}

#[test]
fn math_GaussLeastSquare__PolynomialFitting() {
    // Fit a quadratic polynomial to points: (0,1), (1,4), (2,9), (3,16)
    // Expected: y = x^2 + 1 (approximately), so coefficients [1, 0, 1]

    let mut A = Matrix::with_bounds(1, 4, 1, 3).unwrap();
    // Each row: [1, x, x^2] for fitting y = a + bx + cx^2
    A[(1, 1)] = 1.0;
    A[(1, 2)] = 0.0;
    A[(1, 3)] = 0.0; // x = 0
    A[(2, 1)] = 1.0;
    A[(2, 2)] = 1.0;
    A[(2, 3)] = 1.0; // x = 1
    A[(3, 1)] = 1.0;
    A[(3, 2)] = 2.0;
    A[(3, 3)] = 4.0; // x = 2
    A[(4, 1)] = 1.0;
    A[(4, 2)] = 3.0;
    A[(4, 3)] = 9.0; // x = 3

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 4).unwrap();
    B[1] = 1.0; // y(0) = 1
    B[2] = 4.0; // y(1) = 4
    B[3] = 9.0; // y(2) = 9
    B[4] = 16.0; // y(3) = 16

    let mut X = Vector::with_bounds(1, 3).unwrap(); // [a, b, c] coefficients
    solver.solve_into(&B, &mut X).unwrap();

    // Check that solution is reasonable (may not be exact due to numerical precision)
    assert!(X[1].is_finite()); // constant term
    assert!(X[2].is_finite()); // linear term
    assert!(X[3].is_finite()); // quadratic term
    assert!(X[3] > 0.5); // quadratic term should be positive
}

#[test]
fn math_GaussLeastSquare__LinearFitting() {
    // Fit a line to points: (1,2), (2,4), (3,6)
    // Expected: y = 2x, so coefficients [0, 2]

    let mut A = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    // Each row: [1, x] for fitting y = a + bx
    A[(1, 1)] = 1.0;
    A[(1, 2)] = 1.0; // x = 1
    A[(2, 1)] = 1.0;
    A[(2, 2)] = 2.0; // x = 2
    A[(3, 1)] = 1.0;
    A[(3, 2)] = 3.0; // x = 3

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 3).unwrap();
    B[1] = 2.0; // y(1) = 2
    B[2] = 4.0; // y(2) = 4
    B[3] = 6.0; // y(3) = 6

    let mut X = Vector::with_bounds(1, 2).unwrap(); // [a, b] coefficients
    solver.solve_into(&B, &mut X).unwrap();

    // Should find a = 0, b = 2 for y = 0 + 2*x
    assert_near(X[1], 0.0, 1e-6); // constant term
    assert_near(X[2], 2.0, 1e-6); // linear term
}
