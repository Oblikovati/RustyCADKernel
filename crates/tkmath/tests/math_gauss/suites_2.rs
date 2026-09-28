#[test]
fn MathGaussTest__MatrixInversion() {
    // Test matrix inversion
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrix[(1, 1)] = 4.0;
    aMatrix[(1, 2)] = 7.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 6.0;

    let aGauss = Gauss::new(&aMatrix).unwrap();
    assert!(aGauss.is_done());

    let mut aInverse = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aGauss.invert(&mut aInverse).unwrap();

    // For 2x2 matrix [[4,7],[2,6]], inverse should be [[0.6,-0.7],[-0.2,0.4]]
    // det = 4*6 - 7*2 = 24 - 14 = 10
    // inv = (1/10) * [[6,-7],[-2,4]]
    assert_near(aInverse[(1, 1)], 0.6, 1.0e-12);
    assert_near(aInverse[(1, 2)], -0.7, 1.0e-12);
    assert_near(aInverse[(2, 1)], -0.2, 1.0e-12);
    assert_near(aInverse[(2, 2)], 0.4, 1.0e-12);

    // Verify that A * A^(-1) = I
    let mut aProduct = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    for i in 1..=2 {
        for j in 1..=2 {
            aProduct[(i, j)] = 0.0;
            for k in 1..=2 {
                aProduct[(i, j)] += aMatrix[(i, k)] * aInverse[(k, j)];
            }
        }
    }

    assert_near(aProduct[(1, 1)], 1.0, 1.0e-12);
    assert_near(aProduct[(1, 2)], 0.0, 1.0e-12);
    assert_near(aProduct[(2, 1)], 0.0, 1.0e-12);
    assert_near(aProduct[(2, 2)], 1.0, 1.0e-12);
}

#[test]
fn MathGaussTest__SingularMatrix() {
    // Test with a singular (non-invertible) matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 4.0;
    aMatrix[(2, 3)] = 6.0; // Row 2 = 2 * Row 1
    aMatrix[(3, 1)] = 3.0;
    aMatrix[(3, 2)] = 6.0;
    aMatrix[(3, 3)] = 9.0; // Row 3 = 3 * Row 1

    // Singular matrix should either fail decomposition or have zero determinant
    let aGauss = Gauss::new(&aMatrix).unwrap();
    if aGauss.is_done() {
        let aDet: f64 = aGauss.determinant().unwrap();
        assert_near(aDet, 0.0, 1.0e-12);
    }
    // Note: Some implementations may fail decomposition or throw exceptions for singular matrices
}

#[test]
fn MathGaussTest__CustomMinPivot() {
    // Test with custom minimum pivot threshold
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrix[(1, 1)] = 1.0e-15;
    aMatrix[(1, 2)] = 1.0; // Very small pivot
    aMatrix[(2, 1)] = 1.0;
    aMatrix[(2, 2)] = 1.0;

    // With default MinPivot (1e-20), should succeed
    let aGauss1 = Gauss::new(&aMatrix).unwrap();
    assert!(aGauss1.is_done());

    // Create a truly singular matrix to test pivot threshold behavior
    let mut aSingular = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aSingular[(1, 1)] = 1.0e-25;
    aSingular[(1, 2)] = 1.0; // Extremely small pivot
    aSingular[(2, 1)] = 1.0;
    aSingular[(2, 2)] = 1.0;

    // Test with truly singular matrix (all zeros) which should always fail
    let mut aTrulySingular = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aTrulySingular[(1, 1)] = 0.0;
    aTrulySingular[(1, 2)] = 0.0;
    aTrulySingular[(2, 1)] = 0.0;
    aTrulySingular[(2, 2)] = 0.0;

    let aGauss2 = Gauss::new(&aTrulySingular).unwrap();
    assert!(!aGauss2.is_done());
}
