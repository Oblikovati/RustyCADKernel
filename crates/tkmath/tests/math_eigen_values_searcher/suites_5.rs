#[test]
fn math_EigenValuesSearcherTest__AntisymmetricSubdiagonal() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 5).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 5).unwrap();

    // Symmetric tridiagonal with specific pattern
    for i in 1..=5 {
        aDiagonal.set_value(i, f64::from(i)).unwrap();
    }

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1.0).unwrap();
    aSubdiagonal.set_value(3, -2.0).unwrap();
    aSubdiagonal.set_value(4, 1.0).unwrap();
    aSubdiagonal.set_value(5, -2.0).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

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
    }

    // Verify orthogonality
    for i in 1..=5 {
        for j in i + 1..=5 {
            let vec_i: Vector<'static> = searcher.eigen_vector(i).unwrap();
            let vec_j: Vector<'static> = searcher.eigen_vector(j).unwrap();
            assert!(are_orthogonal(&vec_i, &vec_j, 1e-9));
        }
    }
}

// Test Wilkinson matrix (known challenging case)
#[test]
fn math_EigenValuesSearcherTest__WilkinsonMatrix() {
    let n: i32 = 5;
    let mut aDiagonal = Array1::<f64>::with_bounds(1, n).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, n).unwrap();

    // Wilkinson matrix W_n pattern
    let m: i32 = (n - 1) / 2;
    for i in 1..=n {
        aDiagonal
            .set_value(i, f64::from((i - 1 - m).abs()))
            .unwrap();
    }

    aSubdiagonal.set_value(1, 0.0).unwrap();
    for i in 2..=n {
        aSubdiagonal.set_value(i, 1.0).unwrap();
    }

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    let mut originalMatrix = Matrix::with_bounds(1, n, 1, n).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=n {
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
}

// Test with mixed positive/negative diagonal
#[test]
fn math_EigenValuesSearcherTest__MixedSignDiagonal() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();

    aDiagonal.set_value(1, -2.0).unwrap();
    aDiagonal.set_value(2, 1.0).unwrap();
    aDiagonal.set_value(3, -1.0).unwrap();
    aDiagonal.set_value(4, 3.0).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1.5).unwrap();
    aSubdiagonal.set_value(3, 0.8).unwrap();
    aSubdiagonal.set_value(4, -0.6).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    let mut originalMatrix = Matrix::with_bounds(1, 4, 1, 4).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    let mut eigenvals: Vec<f64> = Vec::new();
    for i in 1..=4 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();

        eigenvals.push(eigenval);
        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-9
        ));
    }

    // Check trace preservation (sum of eigenvalues = sum of diagonal)
    let mut eigenSum: f64 = 0.0;
    let mut diagSum: f64 = 0.0;
    for i in 1..=4 {
        eigenSum += eigenvals[(i - 1) as usize];
        diagSum += aDiagonal[i];
    }
    assert_near(eigenSum, diagSum, 1e-9);
}

// Test maximum size matrix (stress test)
