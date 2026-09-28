#[test]
fn math_EigenValuesSearcherTest__ConstructorDimensionMismatch() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 2).unwrap(); // Wrong size

    aDiagonal.set_value(1, 1.0).unwrap();
    aDiagonal.set_value(2, 2.0).unwrap();
    aDiagonal.set_value(3, 3.0).unwrap();

    aSubdiagonal.set_value(1, 0.5).unwrap();
    aSubdiagonal.set_value(2, 1.5).unwrap();

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    // Should not be done due to dimension mismatch
    assert!(!searcher.is_done());
}

// Test 1x1 matrix (trivial case)
#[test]
fn math_EigenValuesSearcherTest__OneByOneMatrix() {
    let mut aDiagonal = Array1::<f64>::with_bounds(1, 1).unwrap();
    let mut aSubdiagonal = Array1::<f64>::with_bounds(1, 1).unwrap();

    aDiagonal.set_value(1, 5.0).unwrap();
    aSubdiagonal.set_value(1, 0.0).unwrap(); // Not used for 1x1 matrix

    let searcher = EigenValuesSearcher::new(&aDiagonal, &aSubdiagonal).unwrap();

    assert!(searcher.is_done());
    assert_eq!(searcher.dimension(), 1);
    assert_near(searcher.eigen_value(1).unwrap(), 5.0, 1e-7);

    let eigenvec: Vector<'static> = searcher.eigen_vector(1).unwrap();
    assert_eq!(eigenvec.length(), 1);
    assert_near(eigenvec[1], 1.0, 1e-7);
}

// Test 2x2 symmetric tridiagonal matrix
