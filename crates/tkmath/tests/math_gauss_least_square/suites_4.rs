#[test]
fn math_GaussLeastSquare__NoisyData() {
    // Fit line to noisy data around y = x + 1
    let mut A = Matrix::with_bounds(1, 5, 1, 2).unwrap();
    A[(1, 1)] = 1.0;
    A[(1, 2)] = 1.0; // x = 1
    A[(2, 1)] = 1.0;
    A[(2, 2)] = 2.0; // x = 2
    A[(3, 1)] = 1.0;
    A[(3, 2)] = 3.0; // x = 3
    A[(4, 1)] = 1.0;
    A[(4, 2)] = 4.0; // x = 4
    A[(5, 1)] = 1.0;
    A[(5, 2)] = 5.0; // x = 5

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 5).unwrap();
    B[1] = 2.1; // y = 1*1 + 1 = 2 with noise
    B[2] = 2.9; // y = 1*2 + 1 = 3 with noise
    B[3] = 4.1; // y = 1*3 + 1 = 4 with noise
    B[4] = 4.9; // y = 1*4 + 1 = 5 with noise
    B[5] = 6.1; // y = 1*5 + 1 = 6 with noise

    let mut X = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B, &mut X).unwrap();

    // Should find approximately a = 1, b = 1
    assert_near(X[1], 1.0, 0.2); // constant term (with some tolerance for noise)
    assert_near(X[2], 1.0, 0.2); // linear term
}

#[test]
fn math_GaussLeastSquare__ZeroRightHandSide() {
    // Test with zero right-hand side
    let mut A = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    A[(1, 1)] = 1.0;
    A[(1, 2)] = 0.0;
    A[(2, 1)] = 0.0;
    A[(2, 2)] = 1.0;

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    let mut B = Vector::with_bounds(1, 2).unwrap();
    B[1] = 0.0;
    B[2] = 0.0;

    let mut X = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B, &mut X).unwrap();

    assert_near(X[1], 0.0, 1e-6);
    assert_near(X[2], 0.0, 1e-6);
}

#[test]
fn math_GaussLeastSquare__MultipleRightHandSides() {
    // Test solving multiple systems with same matrix
    let mut A = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    A[(1, 1)] = 2.0;
    A[(1, 2)] = 1.0;
    A[(2, 1)] = 1.0;
    A[(2, 2)] = 2.0;

    let solver = GaussLeastSquares::new(&A).unwrap();

    assert!(solver.is_done());

    // First system
    let mut B1 = Vector::with_bounds(1, 2).unwrap();
    B1[1] = 3.0;
    B1[2] = 3.0;

    let mut X1 = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B1, &mut X1).unwrap();

    assert_near(X1[1], 1.0, 1e-6);
    assert_near(X1[2], 1.0, 1e-6);

    // Second system with same matrix
    let mut B2 = Vector::with_bounds(1, 2).unwrap();
    B2[1] = 5.0;
    B2[2] = 4.0;

    let mut X2 = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&B2, &mut X2).unwrap();

    assert_near(X2[1], 2.0, 1e-6);
    assert_near(X2[2], 1.0, 1e-6);
}
