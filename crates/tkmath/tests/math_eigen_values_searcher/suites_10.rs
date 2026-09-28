#[test]
fn math_EigenValuesSearcherTest__DeflationBoundaryCondition() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 4).unwrap();

    // Create diagonal elements of different scales
    aDiagonal.set_value(1, 1.0).unwrap();
    aDiagonal.set_value(2, 1000.0).unwrap();
    aDiagonal.set_value(3, 1e-6).unwrap();
    aDiagonal.set_value(4, 1e6).unwrap();

    aSubdiagonal.set_value(1, 0.0).unwrap();

    // Set subdiagonal elements right at the deflation boundary for different scales
    let mut eps: f64 = f64::EPSILON;

    // For first pair: should deflate
    let mut sum1: f64 = aDiagonal[1].abs() + aDiagonal[2].abs();
    aSubdiagonal.set_value(2, sum1 * eps * 0.1).unwrap(); // Below machine epsilon relative to sum

    // For second pair: should deflate
    let mut sum2: f64 = aDiagonal[2].abs() + aDiagonal[3].abs();
    aSubdiagonal.set_value(3, sum2 * eps * 0.1).unwrap();

    // For third pair: should deflate
    let mut sum3: f64 = aDiagonal[3].abs() + aDiagonal[4].abs();
    aSubdiagonal.set_value(4, sum3 * eps * 0.1).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();
    assert!(searcher.is_done());

    // Verify that algorithm handles the mixed scales correctly
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
        assert!(eigenval.is_finite());
        assert!(!eigenval.is_nan());
    }
}
