#[test]
fn MathLin_LeastSquares_Test__ArbitraryBoundsRankAndDiagnostics() {
    let mut aA: Matrix = Matrix::with_bounds(-2, 1, 5, 6).unwrap();
    for i in 0..4 {
        aA[(-2 + i, 5)] = 1.0;
        aA[(-2 + i, 6)] = (i) as f64;
    }
    let mut aB: Vector = Vector::with_bounds(10, 13).unwrap();
    for i in 0..4 {
        aB[10 + i] = 2.0 + 3.0 * i as f64;
    }
    let aResult = least_squares(&aA, &aB, LeastSquaresMethod::Svd, 1e-15);
    assert!(aResult.is_done());
    assert_eq!(aResult.solution.as_ref().unwrap().lower(), 0);
    assert_eq!(aResult.rank, 2);
    assert!(aResult.condition_number.is_some());
    assert!(aResult.condition_number.unwrap().is_finite());
    near(
        *aResult.solution.as_ref().unwrap().at(0_usize).unwrap(),
        2.0,
        1.0e-12,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1_usize).unwrap(),
        3.0,
        1.0e-12,
    );
    near(aResult.residual.unwrap(), 0.0, 1.0e-12);
}

#[test]
fn MathLin_LeastSquares_Test__GCVSelectionAndInvalidInput() {
    let mut aA: Matrix = Matrix::with_bounds(1, 8, 1, 3).unwrap();
    let mut aB: Vector = Vector::with_bounds(-4, 3).unwrap();
    for i in 1..=8 {
        let mut aX: f64 = (i - 1) as f64 / 7.0;
        aA[(i, 1)] = 1.0;
        aA[(i, 2)] = aX;
        aA[(i, 3)] = aX * aX;
        aB[-5 + i] = 1.0 + 2.0 * aX + (if i % 2 == 0 { 0.3 } else { -0.3 });
    }
    let aLambda: Option<f64> = optimal_regularization(&aA, &aB, 1.0e-8, 1.0e1, 30);
    assert!(aLambda.is_some());
    assert!(aLambda.unwrap().is_finite());
    assert!(aLambda.unwrap() >= 1.0e-8);
    assert!(aLambda.unwrap() <= 1.0e1);
    assert!(!(optimal_regularization(&aA, &aB, -1.0, 1.0, 10).is_some()));

    aB.init(f64::MAX);
    assert!(optimal_regularization(&aA, &aB, 1.0e-8, 1.0e1, 30).is_some());
}
