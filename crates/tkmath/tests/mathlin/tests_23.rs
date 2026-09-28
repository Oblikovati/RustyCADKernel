#[test]
fn MathLin_Test__CompareWithOldAPI_SVD() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 2.0;
    aMat[(1, 3)] = 3.0;
    aMat[(2, 1)] = 4.0;
    aMat[(2, 2)] = 5.0;
    aMat[(2, 3)] = 6.0;
    aMat[(3, 1)] = 7.0;
    aMat[(3, 2)] = 8.0;
    aMat[(3, 3)] = 10.0;

    let mut aB: Vector = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 1.0;
    aB[2] = 2.0;
    aB[3] = 3.0;

    let mut anOldSVD = Svd::new(&aMat).unwrap();
    let mut anOldSol: Vector = Vector::with_bounds(1, 3).unwrap();
    anOldSVD.solve(&aB, &mut anOldSol, 1e-6).unwrap();

    let aNewResult = solve_svd(&aMat, &aB, 1e-06);

    assert!(anOldSVD.is_done());
    assert!(aNewResult.is_done());

    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *anOldSol.at(i).unwrap(),
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            1.0e-8,
        );
    }
}

#[test]
fn MathLin_Test__CompareWithOldAPI_Householder() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 1.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 2.0;
    aMat[(3, 1)] = 1.0;
    aMat[(3, 2)] = 3.0;

    let mut aB: Vector = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 2.0;
    aB[2] = 3.0;
    aB[3] = 4.0;

    let mut aBMat: Matrix = Matrix::with_bounds(1, 3, 1, 1).unwrap();
    aBMat[(1, 1)] = aB[1];
    aBMat[(2, 1)] = aB[2];
    aBMat[(3, 1)] = aB[3];
    let anOldHH = Householder::from_matrix(&aMat, &aBMat, 1e-20).unwrap();

    let aNewResult = solve_qr(&aMat, &aB, 1e-15);

    assert!(anOldHH.is_done());
    assert!(aNewResult.is_done());

    let mut anOldSol: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldHH.value(&mut anOldSol, 1).unwrap();

    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *anOldSol.at(i).unwrap(),
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            1.0e-8,
        );
    }
}
