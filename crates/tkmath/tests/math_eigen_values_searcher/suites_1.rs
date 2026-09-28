#[test]
fn math_EigenValuesSearcherTest__TwoByTwoMatrix() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 2).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 2).unwrap();

    // Matrix: [2 1]
    //         [1 3]
    aDiagonal.set_value(1, 2.0).unwrap();
    aDiagonal.set_value(2, 3.0).unwrap();
    aSubdiagonal.set_value(1, 0.0).unwrap(); // Not used
    aSubdiagonal.set_value(2, 1.0).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());
    assert_eq!(searcher.dimension(), 2);

    // Verify eigenvalues are approximately 0.382 and 4.618
    let mut eigenvals: Vec<f64> = vec![
        searcher.eigen_value(1).unwrap(),
        searcher.eigen_value(2).unwrap(),
    ];
    eigenvals.sort_by(f64::total_cmp);

    let mut lambda1: f64 = 2.5 - 0.5 * 5.0_f64.sqrt(); // ~= 0.382
    let mut lambda2: f64 = 2.5 + 0.5 * 5.0_f64.sqrt(); // ~= 4.618

    assert_near(eigenvals[0], lambda1, 1e-10);
    assert_near(eigenvals[1], lambda2, 1e-10);

    // Create original matrix for verification
    let mut originalMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    // Verify eigenvalue-eigenvector relationships
    for i in 1..=2 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-10
        ));
        assert_near(vector_norm(&eigenvec), 1.0, 1e-10); // Should be normalized
    }

    // Verify eigenvectors are orthogonal
    let vec1: Vector<'static> = searcher.eigen_vector(1).unwrap();
    let vec2: Vector<'static> = searcher.eigen_vector(2).unwrap();
    assert!(are_orthogonal(&vec1, &vec2, 1e-10));
}

// Test 3x3 symmetric tridiagonal matrix
#[test]
fn math_EigenValuesSearcherTest__ThreeByThreeMatrix() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    // Matrix: [4 1 0]
    //         [1 4 1]
    //         [0 1 4]
    aDiagonal.set_value(1, 4.0).unwrap();
    aDiagonal.set_value(2, 4.0).unwrap();
    aDiagonal.set_value(3, 4.0).unwrap();
    aSubdiagonal.set_value(1, 0.0).unwrap(); // Not used
    aSubdiagonal.set_value(2, 1.0).unwrap();
    aSubdiagonal.set_value(3, 1.0).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());
    assert_eq!(searcher.dimension(), 3);

    // Create original matrix for verification
    let mut originalMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    // Verify all eigenvalue-eigenvector pairs
    for i in 1..=3 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-10
        ));
        assert_near(vector_norm(&eigenvec), 1.0, 1e-10);
    }

    // Verify orthogonality of eigenvectors
    for i in 1..=3 {
        for j in i + 1..=3 {
            let vec_i: Vector<'static> = searcher.eigen_vector(i).unwrap();
            let vec_j: Vector<'static> = searcher.eigen_vector(j).unwrap();
            assert!(are_orthogonal(&vec_i, &vec_j, 1e-10));
        }
    }
}

// Test diagonal matrix (eigenvalues should be diagonal elements)
#[test]
fn math_EigenValuesSearcherTest__DiagonalMatrix() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();

    aDiagonal.set_value(1, 1.0).unwrap();
    aDiagonal.set_value(2, 2.0).unwrap();
    aDiagonal.set_value(3, 3.0).unwrap();
    aDiagonal.set_value(4, 4.0).unwrap();

    // All subdiagonal elements are zero
    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 0.0).unwrap();
    aSubdiagonal.set_value(3, 0.0).unwrap();
    aSubdiagonal.set_value(4, 0.0).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());
    assert_eq!(searcher.dimension(), 4);

    // For a diagonal matrix, eigenvalues should be the diagonal elements
    let mut eigenvals: Vec<f64> = Vec::new();
    for i in 1..=4 {
        eigenvals.push(searcher.eigen_value(i).unwrap());
    }

    eigenvals.sort_by(f64::total_cmp);

    assert_near(eigenvals[0], 1.0, 1e-7);
    assert_near(eigenvals[1], 2.0, 1e-7);
    assert_near(eigenvals[2], 3.0, 1e-7);
    assert_near(eigenvals[3], 4.0, 1e-7);
}

// Test identity matrix
