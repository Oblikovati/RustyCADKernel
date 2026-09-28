#[test]
fn MathLin_Jacobi_Test__SpectralDecomposition() {
    let aMat: Matrix = CreateSPD(3);

    let aResult = spectral_decomposition(&aMat);

    assert!(aResult.is_done());

    let aD: &Vector = aResult.eigen_values.as_ref().unwrap();
    let aV: &Matrix = aResult.eigen_vectors.as_ref().unwrap();

    let mut aDiag: Matrix = Matrix::filled(1, 3, 1, 3, 0.0).unwrap();
    for i in 0..aD.length() as usize {
        *aDiag.change_at(i, i).unwrap() = *aD.at(i).unwrap();
    }

    let aVD: Matrix = matmul(aV, &aDiag);
    let aReconstructed: Matrix = matmul(&aVD, &transpose(aV));

    for i in 1..=3 {
        for j in 1..=3 {
            near(aReconstructed[(i, j)], aMat[(i, j)], 1.0e-8);
        }
    }
}

#[test]
fn MathLin_Jacobi_Test__MatrixSqrt() {
    let aMat: Matrix = CreateSPD(2);

    let aSqrt = matrix_sqrt(&aMat);

    assert!(aSqrt.is_some());

    let aCheck: Matrix = matmul(aSqrt.as_ref().unwrap(), aSqrt.as_ref().unwrap());

    for i in 1..=2 {
        for j in 1..=2 {
            near(aCheck[(i, j)], aMat[(i, j)], 1.0e-8);
        }
    }
}
