#[test]
fn MathLin_Test__CompareWithOldAPI_Jacobi() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMat[(1, 1)] = 3.0;
    aMat[(1, 2)] = 1.0;
    aMat[(1, 3)] = 0.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 3.0;
    aMat[(2, 3)] = 1.0;
    aMat[(3, 1)] = 0.0;
    aMat[(3, 2)] = 1.0;
    aMat[(3, 3)] = 3.0;

    let anOldJacobi = Jacobi::new(&aMat).unwrap();

    let aNewResult = jacobi(&aMat, true);

    assert!(anOldJacobi.is_done());
    assert!(aNewResult.is_done());

    let aNewEig: &Vector = aNewResult.eigen_values.as_ref().unwrap();
    for i in 0..aNewEig.length() as usize {
        near(
            anOldJacobi.value((i) as i32 + 1).unwrap(),
            *aNewEig.at(i).unwrap(),
            1.0e-8,
        );
    }
}

#[test]
fn MathLin_Test__CompareWithOldAPI_GaussLeastSquare() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 4, 1, 2).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 1.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 2.0;
    aMat[(3, 1)] = 1.0;
    aMat[(3, 2)] = 3.0;
    aMat[(4, 1)] = 1.0;
    aMat[(4, 2)] = 4.0;

    let mut aB: Vector = Vector::with_bounds(1, 4).unwrap();
    aB[1] = 2.0;
    aB[2] = 3.0;
    aB[3] = 4.0;
    aB[4] = 5.0;

    let anOldLS = GaussLeastSquares::new(&aMat).unwrap();
    let mut anOldSol: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldLS.solve_into(&aB, &mut anOldSol).unwrap();

    let aNewResult = least_squares(&aMat, &aB, LeastSquaresMethod::Qr, 1e-15);

    assert!(anOldLS.is_done());
    assert!(aNewResult.is_done());

    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *anOldSol.at(i).unwrap(),
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            1.0e-8,
        );
    }
}
