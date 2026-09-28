#[test]
fn math_EigenValuesSearcherTest__SmallSubdiagonalElements() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    aDiagonal.set_value(1, 1.0).unwrap();
    aDiagonal.set_value(2, 2.0).unwrap();
    aDiagonal.set_value(3, 3.0).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1e-14).unwrap();
    aSubdiagonal.set_value(3, 1e-14).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Should converge to approximately diagonal values
    let mut eigenvals: Vec<f64> = Vec::new();
    for i in 1..=3 {
        eigenvals.push(searcher.eigen_value(i).unwrap());
    }

    eigenvals.sort_by(f64::total_cmp);

    assert_near(eigenvals[0], 1.0, 1e-10);
    assert_near(eigenvals[1], 2.0, 1e-10);
    assert_near(eigenvals[2], 3.0, 1e-10);
}

// Test with zero diagonal elements
#[test]
fn math_EigenValuesSearcherTest__ZeroDiagonalElements() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    // Matrix: [0 1 0]
    //         [1 0 1]
    //         [0 1 0]
    aDiagonal.set_value(1, 0.0).unwrap();
    aDiagonal.set_value(2, 0.0).unwrap();
    aDiagonal.set_value(3, 0.0).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1.0).unwrap();
    aSubdiagonal.set_value(3, 1.0).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());
    assert_eq!(searcher.dimension(), 3);

    // Eigenvalues should be -sqrt(2), 0, sqrt(2)
    let mut eigenvals: Vec<f64> = Vec::new();
    for i in 1..=3 {
        eigenvals.push(searcher.eigen_value(i).unwrap());
    }

    eigenvals.sort_by(f64::total_cmp);

    assert_near(eigenvals[0], -2.0_f64.sqrt(), 1e-10);
    assert_near(eigenvals[1], 0.0, 1e-10);
    assert_near(eigenvals[2], 2.0_f64.sqrt(), 1e-10);
}

// Test with large diagonal elements (numerical stability)
#[test]
fn math_EigenValuesSearcherTest__LargeDiagonalElements() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    aDiagonal.set_value(1, 1e6).unwrap();
    aDiagonal.set_value(2, 2e6).unwrap();
    aDiagonal.set_value(3, 3e6).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1e3).unwrap();
    aSubdiagonal.set_value(3, 2e3).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Create original matrix for verification
    let mut originalMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=3 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-6
        ));
        assert_near(vector_norm(&eigenvec), 1.0, 1e-10);
    }
}

// Test with alternating pattern
