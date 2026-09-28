#[test]
fn math_EigenValuesSearcherTest__PathologicalEqualElements() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 5).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 5).unwrap();

    // All elements equal - degenerate case
    let mut value: f64 = 42.0;
    for i in 1..=5 {
        aDiagonal.set_value(i, value).unwrap();
    }

    aSubdiagonal.set_value(1, 0.0).unwrap();
    for i in 2..=5 {
        aSubdiagonal.set_value(i, value).unwrap();
    }

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Algorithm should still work correctly
    let mut originalMatrix = Matrix::with_bounds(1, 5, 1, 5).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

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
}

// Test matrix with systematically increasing subdiagonal elements
#[test]
fn math_EigenValuesSearcherTest__IncreasingSubdiagonal() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 6).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 6).unwrap();

    for i in 1..=6 {
        aDiagonal.set_value(i, 1.0).unwrap();
    }

    aSubdiagonal.set_value(1, 0.0).unwrap();
    for i in 2..=6 {
        aSubdiagonal.set_value(i, f64::from(i - 1) * 0.1).unwrap();
    }

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    let mut originalMatrix = Matrix::with_bounds(1, 6, 1, 6).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    // Verify all pairs and orthogonality
    for i in 1..=6 {
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

    // Check orthogonality of all eigenvector pairs
    for i in 1..=6 {
        for j in i + 1..=6 {
            let vec_i: Vector<'static> = searcher.eigen_vector(i).unwrap();
            let vec_j: Vector<'static> = searcher.eigen_vector(j).unwrap();
            assert!(are_orthogonal(&vec_i, &vec_j, 1e-10));
        }
    }
}

// Test to demonstrate and verify the deflation condition behavior
// This test documents the precision semantics of the deflation test:
// |e[i]| + (|d[i]| + |d[i+1]|) == |d[i]| + |d[i+1]| in floating-point arithmetic
#[test]
fn math_EigenValuesSearcherTest__DeflationConditionSemantics() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    // Test 1: Large diagonal elements with subdiagonal at machine epsilon level
    let mut largeDiag1: f64 = 1e8;
    let mut largeDiag2: f64 = 2e8;
    let mut machEpsilonLevel: f64 = largeDiag1 * f64::EPSILON;

    aDiagonal.set_value(1, largeDiag1).unwrap();
    aDiagonal.set_value(2, largeDiag2).unwrap();
    aDiagonal.set_value(3, 3e8).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, machEpsilonLevel).unwrap(); // Should trigger deflation due to floating-point precision
    aSubdiagonal.set_value(3, machEpsilonLevel * 0.5).unwrap(); // Even smaller, should definitely deflate

    // Verify the mathematical behavior that the deflation condition tests
    let mut diagSum: f64 = largeDiag1.abs() + largeDiag2.abs();
    let mut testCondition: f64 = machEpsilonLevel + diagSum;

    // This should be true in floating-point arithmetic due to precision limits
    assert_eq!(testCondition, diagSum);

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();
    assert!(searcher.is_done());

    // Should converge efficiently due to deflation
    let mut originalMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    create_tridiagonal_matrix(&aDiagonal, &aSubdiagonal, &mut originalMatrix);

    for i in 1..=3 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();
        // Use looser tolerance for very large eigenvalues
        assert!(verify_eigen_pair(
            &originalMatrix,
            eigenval,
            &eigenvec,
            1e-6
        ));
    }
}

// Test edge case where deflation condition is exactly at the boundary
