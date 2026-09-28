#[test]
fn math_EigenValuesSearcherTest__NumericalStability() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    // Mix of very large and very small values
    aDiagonal.set_value(1, 1e-100).unwrap();
    aDiagonal.set_value(2, 1e100).unwrap();
    aDiagonal.set_value(3, 1e-50).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1e-75).unwrap();
    aSubdiagonal.set_value(3, 1e75).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());

    // Should handle extreme values without overflow/underflow
    for i in 1..=3 {
        let mut eigenval: f64 = searcher.eigen_value(i).unwrap();
        assert!(eigenval.is_finite());
        assert!(!eigenval.is_nan());

        let eigenvec: Vector<'static> = searcher.eigen_vector(i).unwrap();
        for j in 1..=3 {
            assert!(eigenvec[j].is_finite());
            assert!(!eigenvec[j].is_nan());
        }
    }
}

// Test Wilkinson shift effectiveness
#[test]
fn math_EigenValuesSearcherTest__WilkinsonShiftAccuracy() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();

    // Matrix where Wilkinson shift should provide fast convergence
    aDiagonal.set_value(1, 1.0).unwrap();
    aDiagonal.set_value(2, 2.0).unwrap();
    aDiagonal.set_value(3, 1.0000001).unwrap(); // Very close to first diagonal element

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1.0).unwrap();
    aSubdiagonal.set_value(3, 0.0000001).unwrap(); // Very small

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
            1e-10
        ));
    }
}

// Test special case when radius becomes zero in QL step
#[test]
fn math_EigenValuesSearcherTest__ZeroRadiusHandling() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();

    // Configuration that might lead to zero radius in computeHypotenuseLength
    aDiagonal.set_value(1, 0.0).unwrap();
    aDiagonal.set_value(2, 0.0).unwrap();
    aDiagonal.set_value(3, 1.0).unwrap();
    aDiagonal.set_value(4, 1.0).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();
    aSubdiagonal.set_value(2, 1e-16).unwrap(); // Very small but non-zero
    aSubdiagonal.set_value(3, 1e-16).unwrap();
    aSubdiagonal.set_value(4, 0.0).unwrap();

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
    }
}

// Test pathological case with all equal elements
