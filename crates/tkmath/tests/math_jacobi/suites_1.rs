#[test]
fn MathJacobiTest__SimpleSymmetricMatrix() {
    // Test with simple 2x2 symmetric matrix
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrix[(1, 1)] = 3.0;
    aMatrix[(1, 2)] = 1.0;
    aMatrix[(2, 1)] = 1.0;
    aMatrix[(2, 2)] = 3.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    assert!(aJacobi.is_done());

    let aValues = aJacobi.values().unwrap();

    // For this matrix, eigenvalues are 2 and 4
    let mut aEigenvals = [0.0; 2];
    aEigenvals[0] = aValues[1];
    aEigenvals[1] = aValues[2];
    aEigenvals.sort_by(f64::total_cmp);

    assert_near(aEigenvals[0], 2.0, 1.0e-10);
    assert_near(aEigenvals[1], 4.0, 1.0e-10);
}

#[test]
fn MathJacobiTest__EigenvectorVerification() {
    // Test eigenvector computation and verification
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrix[(1, 1)] = 4.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 1.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    assert!(aJacobi.is_done());

    // Verify A * v = lambda * v for each eigenpair
    for i in 1..=2 {
        let mut aV = Vector::with_bounds(1, 2).unwrap();
        aJacobi.vector(i, &mut aV).unwrap();

        let aLambda: f64 = aJacobi.value(i).unwrap();

        // Compute A * v
        let mut aAv = Vector::with_bounds(1, 2).unwrap();
        aAv[1] = aMatrix[(1, 1)] * aV[1] + aMatrix[(1, 2)] * aV[2];
        aAv[2] = aMatrix[(2, 1)] * aV[1] + aMatrix[(2, 2)] * aV[2];

        // Compute lambda * v
        let mut aLambdaV = Vector::with_bounds(1, 2).unwrap();
        aLambdaV[1] = aLambda * aV[1];
        aLambdaV[2] = aLambda * aV[2];

        // Check A*v = lambda*v
        assert_near(aAv[1], aLambdaV[1], 1.0e-10);
        assert_near(aAv[2], aLambdaV[2], 1.0e-10);
    }
}

#[test]
fn MathJacobiTest__LargerSymmetricMatrix() {
    // Test with larger 4x4 symmetric matrix
    let mut aMatrix = Matrix::with_bounds(1, 4, 1, 4).unwrap();
    // Create a symmetric positive definite matrix
    aMatrix[(1, 1)] = 4.0;
    aMatrix[(1, 2)] = 1.0;
    aMatrix[(1, 3)] = 0.5;
    aMatrix[(1, 4)] = 0.2;
    aMatrix[(2, 1)] = 1.0;
    aMatrix[(2, 2)] = 5.0;
    aMatrix[(2, 3)] = 1.5;
    aMatrix[(2, 4)] = 0.3;
    aMatrix[(3, 1)] = 0.5;
    aMatrix[(3, 2)] = 1.5;
    aMatrix[(3, 3)] = 6.0;
    aMatrix[(3, 4)] = 2.0;
    aMatrix[(4, 1)] = 0.2;
    aMatrix[(4, 2)] = 0.3;
    aMatrix[(4, 3)] = 2.0;
    aMatrix[(4, 4)] = 3.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    assert!(aJacobi.is_done());

    let aValues = aJacobi.values().unwrap();
    assert_eq!(aValues.length(), 4);

    // All eigenvalues should be real and positive (positive definite matrix)
    for i in 1..=4 {
        assert!(aJacobi.value(i).unwrap() > 0.0);
    }

    // Verify trace preservation (sum of eigenvalues = sum of diagonal elements)
    let aTraceOriginal: f64 = aMatrix[(1, 1)] + aMatrix[(2, 2)] + aMatrix[(3, 3)] + aMatrix[(4, 4)];
    let aTraceEigenvalues: f64 = aValues[1] + aValues[2] + aValues[3] + aValues[4];

    assert_near(aTraceEigenvalues, aTraceOriginal, 1.0e-10);
}
