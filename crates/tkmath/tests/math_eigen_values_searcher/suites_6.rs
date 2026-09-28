#[test]
fn math_EigenValuesSearcherTest__LargerMatrix() {
    let n: i32 = 8;
    let mut aDiagonal = Array1::<f64>::with_bounds(1, n).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, n).unwrap();

    // Create a more complex pattern
    for i in 1..=n {
        aDiagonal.set_value(i, f64::from(i * i)).unwrap();
    }

    aSubdiagonal.set_value(1, 0.0).unwrap();
    for i in 2..=n {
        let mut val: f64 = if i % 3 == 0 { -1.0 } else { 1.0 };
        aSubdiagonal
            .set_value(i, val * f64::from(i).sqrt())
            .unwrap();
    }

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    let mut originalMatrix = Matrix::with_bounds(1, n, 1, n).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    // Verify all eigenvalue-eigenvector pairs
    for i in 1..=n {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-8
        ));
        assert_near(vector_norm(&eigenvec), 1.0, 1e-8);
    }

    // Test orthogonality for larger matrix
    for i in 1..=n {
        for j in i + 1..=n {
            let vec_i: Vector<'static> = searcher.eigen_vector(i).unwrap();
            let vec_j: Vector<'static> = searcher.eigen_vector(j).unwrap();
            assert!(are_orthogonal(&vec_i, &vec_j, 1e-8));
        }
    }
}

// Test with rational number patterns
#[test]
fn math_EigenValuesSearcherTest__RationalNumberPattern() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();

    aDiagonal.set_value(1, 1.0 / 3.0).unwrap();
    aDiagonal.set_value(2, 2.0 / 3.0).unwrap();
    aDiagonal.set_value(3, 1.0).unwrap();
    aDiagonal.set_value(4, 4.0 / 3.0).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1.0 / 6.0).unwrap();
    aSubdiagonal.set_value(3, 1.0 / 4.0).unwrap();
    aSubdiagonal.set_value(4, 1.0 / 5.0).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    let mut originalMatrix = Matrix::with_bounds(1, 4, 1, 4).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=4 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-12
        ));
        assert_near(vector_norm(&eigenvec), 1.0, 1e-12);
    }
}

// Test near-degenerate case (eigenvalues very close)
#[test]
fn math_EigenValuesSearcherTest__NearDegenerateEigenvalues() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    aDiagonal.set_value(1, 1.0).unwrap();
    aDiagonal.set_value(2, 1.0 + 1e-8).unwrap();
    aDiagonal.set_value(3, 1.0 + 2e-8).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1e-10).unwrap();
    aSubdiagonal.set_value(3, 1e-10).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Should still find distinct eigenvalues despite near-degeneracy
    let mut eigenvals: Vec<f64> = Vec::new();
    for i in 1..=3 {
        eigenvals.push(searcher.eigen_value(i).unwrap());
    }

    eigenvals.sort_by(f64::total_cmp);

    // Eigenvalues should be close to but distinct from diagonal values
    assert_near(eigenvals[0], 1.0, 1e-7);
    assert_near(eigenvals[1], 1.0 + 1e-8, 1e-7);
    assert_near(eigenvals[2], 1.0 + 2e-8, 1e-7);

    let mut originalMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=3 {
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

// Test deflation condition edge case - subdiagonal element smaller than machine epsilon
