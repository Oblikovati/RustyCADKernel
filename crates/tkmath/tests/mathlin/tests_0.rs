#[test]
fn MathLin_SVD_Test__BasicDecomposition_2x2() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMat[(1, 1)] = 3.0;
    aMat[(1, 2)] = 2.0;
    aMat[(2, 1)] = 2.0;
    aMat[(2, 2)] = 3.0;

    let aResult = svd(&aMat, 1e-15);

    assert!(aResult.is_done());
    assert_eq!(aResult.rank, 2);

    let aU: &Matrix = aResult.u.as_ref().unwrap();
    let aS: &Vector = aResult.singular_values.as_ref().unwrap();
    let aV: &Matrix = aResult.v.as_ref().unwrap();

    let mut aSigma: Matrix = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    aSigma[(1, 1)] = *aS.at(0_usize).unwrap();
    aSigma[(2, 2)] = *aS.at(1_usize).unwrap();

    let aUSigma: Matrix = matmul(aU, &aSigma);
    let aVt: Matrix = transpose(aV);
    let aReconstructed: Matrix = matmul(&aUSigma, &aVt);

    for i in 1..=2 {
        for j in 1..=2 {
            near(aReconstructed[(i, j)], aMat[(i, j)], THE_TOLERANCE);
        }
    }
}

#[test]
fn MathLin_SVD_Test__SingularValues() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 2.0;
    aMat[(1, 3)] = 3.0;
    aMat[(2, 1)] = 4.0;
    aMat[(2, 2)] = 5.0;
    aMat[(2, 3)] = 6.0;
    aMat[(3, 1)] = 7.0;
    aMat[(3, 2)] = 8.0;
    aMat[(3, 3)] = 9.0;

    let aResult = svd(&aMat, 1e-15);

    assert!(aResult.is_done());

    let aS: &Vector = aResult.singular_values.as_ref().unwrap();
    for i in 0..aS.length() as usize - 1 {
        assert!(*aS.at(i).unwrap() >= 0.0);
        assert!(*aS.at(i).unwrap() >= *aS.at(i + 1).unwrap());
    }

    assert!(aResult.rank <= 2);
}
