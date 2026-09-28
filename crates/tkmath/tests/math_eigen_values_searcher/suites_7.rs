#[test]
fn math_EigenValuesSearcherTest__DeflationConditionPrecision() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();

    // Large diagonal elements
    aDiagonal.set_value(1, 1e6).unwrap();
    aDiagonal.set_value(2, 2e6).unwrap();
    aDiagonal.set_value(3, 3e6).unwrap();
    aDiagonal.set_value(4, 4e6).unwrap();

    // Subdiagonal elements at machine epsilon level
    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1e-15).unwrap(); // Should trigger deflation
    aSubdiagonal.set_value(3, 2e-15).unwrap(); // Should trigger deflation
    aSubdiagonal.set_value(4, 3e-15).unwrap(); // Should trigger deflation

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Should converge quickly due to deflation
    let mut originalMatrix = Matrix::with_bounds(1, 4, 1, 4).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=4 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-9
        ));
    }

    // Eigenvalues should be close to diagonal elements due to small off-diagonal
    let mut eigenvals: Vec<f64> = Vec::new();
    for i in 1..=4 {
        eigenvals.push(searcher.eigen_value(i).unwrap());
    }

    eigenvals.sort_by(f64::total_cmp);

    assert_near(eigenvals[0], 1e6, 1e3);
    assert_near(eigenvals[1], 2e6, 1e3);
    assert_near(eigenvals[2], 3e6, 1e3);
    assert_near(eigenvals[3], 4e6, 1e3);
}

// Test exact zero subdiagonal elements (should deflate immediately)
#[test]
fn math_EigenValuesSearcherTest__ExactZeroSubdiagonal() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 5).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 5).unwrap();

    aDiagonal.set_value(1, 1.0).unwrap();
    aDiagonal.set_value(2, 4.0).unwrap();
    aDiagonal.set_value(3, 9.0).unwrap();
    aDiagonal.set_value(4, 16.0).unwrap();
    aDiagonal.set_value(5, 25.0).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1.0).unwrap();
    aSubdiagonal.set_value(3, 0.0).unwrap(); // Exact zero - should cause deflation
    aSubdiagonal.set_value(4, 2.0).unwrap();
    aSubdiagonal.set_value(5, 0.0).unwrap(); // Exact zero - should cause deflation

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Should handle block structure correctly
    let mut originalMatrix = Matrix::with_bounds(1, 5, 1, 5).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=5 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-12
        ));
    }
}

// Test convergence behavior with maximum iterations edge case
#[test]
fn math_EigenValuesSearcherTest__ConvergenceBehavior() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 6).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 6).unwrap();

    // Create a matrix that might require more iterations to converge
    for i in 1..=6 {
        aDiagonal.set_value(i, f64::from(i) * 0.1).unwrap();
    }

    aSubdiagonal.set_value(1, 0.0).unwrap();
    for i in 2..=6 {
        aSubdiagonal.set_value(i, 0.99).unwrap(); // Large off-diagonal elements
    }

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Should still converge within maximum iterations
    let mut originalMatrix = Matrix::with_bounds(1, 6, 1, 6).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=6 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-9
        ));
    }
}

// Test underflow/overflow protection
