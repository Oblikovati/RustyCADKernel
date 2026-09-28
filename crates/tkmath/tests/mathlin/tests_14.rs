#[test]
fn MathLin_Jacobi_Test__Eigenvalues_Diagonal() {
    let mut aMat: Matrix = Matrix::filled(1, 3, 1, 3, 0.0).unwrap();
    aMat[(1, 1)] = 3.0;
    aMat[(2, 2)] = 1.0;
    aMat[(3, 3)] = 2.0;

    let aResult = jacobi(&aMat, true);

    assert!(aResult.is_done());

    let aEigenVals: &Vector = aResult.eigen_values.as_ref().unwrap();

    near(*aEigenVals.at(0_usize).unwrap(), 3.0, THE_TOLERANCE);
    near(*aEigenVals.at(1_usize).unwrap(), 2.0, THE_TOLERANCE);
    near(*aEigenVals.at(2_usize).unwrap(), 1.0, THE_TOLERANCE);
}

#[test]
fn MathLin_Jacobi_Test__Eigenvalues_Symmetric() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMat[(1, 1)] = 3.0;
    aMat[(1, 2)] = 1.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 3.0;

    let aResult = jacobi(&aMat, true);

    assert!(aResult.is_done());

    let aEigenVals: &Vector = aResult.eigen_values.as_ref().unwrap();

    near(*aEigenVals.at(0_usize).unwrap(), 4.0, THE_TOLERANCE);
    near(*aEigenVals.at(1_usize).unwrap(), 2.0, THE_TOLERANCE);
}
