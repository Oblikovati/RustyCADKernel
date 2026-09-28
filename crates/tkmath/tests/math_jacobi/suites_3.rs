#[test]
fn MathJacobiTest__OrthogonalityOfEigenvectors() {
    // Test orthogonality of eigenvectors for symmetric matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 6.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 1.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 3.0;
    aMatrix[(2, 3)] = 1.0;
    aMatrix[(3, 1)] = 1.0;
    aMatrix[(3, 2)] = 1.0;
    aMatrix[(3, 3)] = 1.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    assert!(aJacobi.is_done());

    // Get eigenvectors
    let mut aV1 = Vector::with_bounds(1, 3).unwrap();
    let mut aV2 = Vector::with_bounds(1, 3).unwrap();
    let mut aV3 = Vector::with_bounds(1, 3).unwrap();
    aJacobi.vector(1, &mut aV1).unwrap();
    aJacobi.vector(2, &mut aV2).unwrap();
    aJacobi.vector(3, &mut aV3).unwrap();

    // Check orthogonality (dot products should be zero for different eigenvalues)
    let aDot12: f64 = aV1[1] * aV2[1] + aV1[2] * aV2[2] + aV1[3] * aV2[3];
    let aDot13: f64 = aV1[1] * aV3[1] + aV1[2] * aV3[2] + aV1[3] * aV3[3];
    let aDot23: f64 = aV2[1] * aV3[1] + aV2[2] * aV3[2] + aV2[3] * aV3[3];

    let aValues = aJacobi.values().unwrap();

    // Only check orthogonality for distinct eigenvalues
    if (aValues[1] - aValues[2]).abs() > 1.0e-10 {
        assert_near(aDot12, 0.0, 1.0e-10);
    }
    if (aValues[1] - aValues[3]).abs() > 1.0e-10 {
        assert_near(aDot13, 0.0, 1.0e-10);
    }
    if (aValues[2] - aValues[3]).abs() > 1.0e-10 {
        assert_near(aDot23, 0.0, 1.0e-10);
    }
}

#[test]
fn MathJacobiTest__NormalizationOfEigenvectors() {
    // Test that eigenvectors are normalized
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 2.0;
    aMatrix[(1, 2)] = 1.0;
    aMatrix[(1, 3)] = 0.0;
    aMatrix[(2, 1)] = 1.0;
    aMatrix[(2, 2)] = 2.0;
    aMatrix[(2, 3)] = 1.0;
    aMatrix[(3, 1)] = 0.0;
    aMatrix[(3, 2)] = 1.0;
    aMatrix[(3, 3)] = 2.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    assert!(aJacobi.is_done());

    // Check that each eigenvector has unit norm
    for i in 1..=3 {
        let mut aV = Vector::with_bounds(1, 3).unwrap();
        aJacobi.vector(i, &mut aV).unwrap();

        let aNorm: f64 = (aV[1] * aV[1] + aV[2] * aV[2] + aV[3] * aV[3]).sqrt();
        assert_near(aNorm, 1.0, 1.0e-10);
    }
}

#[test]
fn MathJacobiTest__CustomBounds() {
    // Test with custom matrix bounds
    let mut aMatrix = Matrix::with_bounds(2, 3, 2, 3).unwrap();
    aMatrix[(2, 2)] = 5.0;
    aMatrix[(2, 3)] = 1.0;
    aMatrix[(3, 2)] = 1.0;
    aMatrix[(3, 3)] = 3.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    assert!(aJacobi.is_done());

    let aValues = aJacobi.values().unwrap();
    assert_eq!(aValues.length(), 2);

    // Both eigenvalues should be positive
    assert!(aValues[1] > 0.0);
    assert!(aValues[2] > 0.0);
}
