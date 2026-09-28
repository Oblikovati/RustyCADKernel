#[test]
fn MathLin_Jacobi_Test__MismatchedBounds() {
    let mut aA: Matrix = Matrix::with_bounds(-1, 0, 5, 6).unwrap();
    aA[(-1, 5)] = 3.0e200;
    aA[(-1, 6)] = 1.0e200;
    aA[(0, 5)] = 1.0e200;
    aA[(0, 6)] = 3.0e200;
    let aEigen = jacobi(&aA, true);
    assert!(aEigen.is_done());
    near(
        *aEigen.eigen_values.as_ref().unwrap().at(0_usize).unwrap() / 1.0e200,
        4.0,
        1.0e-12,
    );
    near(
        *aEigen.eigen_values.as_ref().unwrap().at(1_usize).unwrap() / 1.0e200,
        2.0,
        1.0e-12,
    );
    let aSqrt: Option<Matrix> = matrix_sqrt(&aA);
    assert!(aSqrt.is_some());
    assert_eq!(aSqrt.as_ref().unwrap().lower_row(), 0);
    assert_eq!(aSqrt.as_ref().unwrap().lower_col(), 0);
}

#[test]
fn MathLin_Jacobi_Test__SubnormalScalingAndInvalidMatrixPowers() {
    let mut aSubnormal: f64 = f64::from_bits(1);
    let mut aMatrix: Matrix = Matrix::filled(0, 1 - 1, 0, 1 - 1, aSubnormal).unwrap();

    let aEigen = jacobi(&aMatrix, true);
    assert!(aEigen.is_done());
    assert_eq!(
        *aEigen.eigen_values.as_ref().unwrap().at(0_usize).unwrap(),
        aSubnormal
    );

    let aSVD = svd(&aMatrix, 1e-15);
    assert!(aSVD.is_done());
    assert_eq!(
        *aSVD.singular_values.as_ref().unwrap().at(0_usize).unwrap(),
        aSubnormal
    );

    *aMatrix.change_at(0_usize, 0_usize).unwrap() = f64::MAX;
    assert!(!(matrix_power(&aMatrix, 2.0).is_some()));
    *aMatrix.change_at(0_usize, 0_usize).unwrap() = 0.0;
    assert!(!(matrix_inv_sqrt(&aMatrix).is_some()));
}
