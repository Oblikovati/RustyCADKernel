#[test]
fn math_EigenValuesSearcherTest__IdentityMatrix() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    // Identity matrix
    aDiagonal.set_value(1, 1.0).unwrap();
    aDiagonal.set_value(2, 1.0).unwrap();
    aDiagonal.set_value(3, 1.0).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 0.0).unwrap();
    aSubdiagonal.set_value(3, 0.0).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());
    assert_eq!(searcher.dimension(), 3);

    // All eigenvalues should be 1.0
    for i in 1..=3 {
        assert_near(searcher.eigen_value(i).unwrap(), 1.0, 1e-7);
    }
}

// Test matrix with negative eigenvalues
#[test]
fn math_EigenValuesSearcherTest__NegativeEigenvalues() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 2).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 2).unwrap();

    // Matrix: [-1  2]
    //         [ 2 -1]
    aDiagonal.set_value(1, -1.0).unwrap();
    aDiagonal.set_value(2, -1.0).unwrap();
    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 2.0).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Eigenvalues should be -3 and 1
    let mut eigenvals: Vec<f64> = vec![
        searcher.eigen_value(1).unwrap(),
        searcher.eigen_value(2).unwrap(),
    ];
    eigenvals.sort_by(f64::total_cmp);

    assert_near(eigenvals[0], -3.0, 1e-10);
    assert_near(eigenvals[1], 1.0, 1e-10);

    // Create original matrix for verification
    let mut originalMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=2 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();
        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-10
        ));
    }
}

// Test larger matrix (5x5)
#[test]
fn math_EigenValuesSearcherTest__FiveByFiveMatrix() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 5).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 5).unwrap();

    // Create a symmetric tridiagonal matrix with pattern
    for i in 1..=5 {
        aDiagonal.set_value(i, 2.0).unwrap();
    }

    aSubdiagonal.set_value(1, 0.0).unwrap(); // Not used
    for i in 2..=5 {
        aSubdiagonal.set_value(i, -1.0).unwrap();
    }

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());
    assert_eq!(searcher.dimension(), 5);

    // Create original matrix for verification
    let mut originalMatrix = Matrix::with_bounds(1, 5, 1, 5).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    // Verify all eigenvalue-eigenvector pairs
    for i in 1..=5 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-9
        ));
        assert_near(vector_norm(&eigenvec), 1.0, 1e-9);
    }

    // Check that all eigenvalues are reasonable
    for i in 1..=5 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        assert!(eigenval >= 0.0); // All eigenvalues should be non-negative for this matrix
        assert!(eigenval <= 4.0); // Should be bounded
    }
}

// Test with very small subdiagonal elements (near singular)
