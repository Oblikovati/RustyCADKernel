#[test]
fn MathJacobiTest__IdentityMatrix() {
    // Test with identity matrix - all eigenvalues should be 1
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 0.0;
    aMatrix[(1, 3)] = 0.0;
    aMatrix[(2, 1)] = 0.0;
    aMatrix[(2, 2)] = 1.0;
    aMatrix[(2, 3)] = 0.0;
    aMatrix[(3, 1)] = 0.0;
    aMatrix[(3, 2)] = 0.0;
    aMatrix[(3, 3)] = 1.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    assert!(aJacobi.is_done());

    let aValues = aJacobi.values().unwrap();
    assert_eq!(aValues.length(), 3);

    // All eigenvalues should be 1
    assert_near(aValues[1], 1.0, 1.0e-10);
    assert_near(aValues[2], 1.0, 1.0e-10);
    assert_near(aValues[3], 1.0, 1.0e-10);

    // Test individual value access
    assert_near(aJacobi.value(1).unwrap(), 1.0, 1.0e-10);
    assert_near(aJacobi.value(2).unwrap(), 1.0, 1.0e-10);
    assert_near(aJacobi.value(3).unwrap(), 1.0, 1.0e-10);
}

#[test]
fn MathJacobiTest__DiagonalMatrix() {
    // Test with diagonal matrix - eigenvalues should be diagonal elements
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 5.0;
    aMatrix[(1, 2)] = 0.0;
    aMatrix[(1, 3)] = 0.0;
    aMatrix[(2, 1)] = 0.0;
    aMatrix[(2, 2)] = 3.0;
    aMatrix[(2, 3)] = 0.0;
    aMatrix[(3, 1)] = 0.0;
    aMatrix[(3, 2)] = 0.0;
    aMatrix[(3, 3)] = 7.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    assert!(aJacobi.is_done());

    let aValues = aJacobi.values().unwrap();

    // Eigenvalues might be in different order, so collect them
    let mut aEigenvals = [0.0; 3];
    aEigenvals[0] = aValues[1];
    aEigenvals[1] = aValues[2];
    aEigenvals[2] = aValues[3];

    aEigenvals.sort_by(f64::total_cmp);

    assert_near(aEigenvals[0], 3.0, 1.0e-10);
    assert_near(aEigenvals[1], 5.0, 1.0e-10);
    assert_near(aEigenvals[2], 7.0, 1.0e-10);
}
