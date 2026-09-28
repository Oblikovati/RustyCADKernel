#[test]
fn math_EigenValuesSearcherTest__AlternatingPattern() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();

    // Matrix with alternating diagonal pattern
    aDiagonal.set_value(1, 1.0).unwrap();
    aDiagonal.set_value(2, -1.0).unwrap();
    aDiagonal.set_value(3, 1.0).unwrap();
    aDiagonal.set_value(4, -1.0).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 0.5).unwrap();
    aSubdiagonal.set_value(3, -0.5).unwrap();
    aSubdiagonal.set_value(4, 0.5).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Verify eigenvalue-eigenvector relationships
    let mut originalMatrix = Matrix::with_bounds(1, 4, 1, 4).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=4 {
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

// Test repeated eigenvalues (multiple eigenvalues)
#[test]
fn math_EigenValuesSearcherTest__RepeatedEigenvalues() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();

    // Matrix designed to have repeated eigenvalues
    aDiagonal.set_value(1, 2.0).unwrap();
    aDiagonal.set_value(2, 2.0).unwrap();
    aDiagonal.set_value(3, 2.0).unwrap();
    aDiagonal.set_value(4, 2.0).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 0.0).unwrap(); // Block diagonal structure
    aSubdiagonal.set_value(3, 0.0).unwrap();
    aSubdiagonal.set_value(4, 0.0).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // All eigenvalues should be 2.0
    for i in 1..=4 {
        assert_near(searcher.eigen_value(i).unwrap(), 2.0, 1e-7);
    }
}

// Test with very small matrix elements (precision test)
#[test]
fn math_EigenValuesSearcherTest__VerySmallElements() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    aDiagonal.set_value(1, 1e-10).unwrap();
    aDiagonal.set_value(2, 2e-10).unwrap();
    aDiagonal.set_value(3, 3e-10).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1e-11).unwrap();
    aSubdiagonal.set_value(3, 2e-11).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    let mut originalMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=3 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-18
        ));
        assert_near(vector_norm(&eigenvec), 1.0, 1e-12);
    }
}

// Test antisymmetric subdiagonal pattern
