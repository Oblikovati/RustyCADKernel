#[test]
fn MathLin_LeastSquares_Test__ZeroRegularizationUsesSVDCutoff() {
    let mut aA: Matrix = Matrix::filled(0, 2 - 1, 0, 2 - 1, 0.0).unwrap();
    *aA.change_at(0_usize, 0_usize).unwrap() = 1.0;
    *aA.change_at(1_usize, 1_usize).unwrap() = f64::EPSILON;
    let mut aB: Vector = Vector::filled(0, 2 - 1, 1.0).unwrap();

    let aResult = regularized_least_squares(&aA, &aB, 0.0, 0.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.rank, 1);
    assert_eq!(
        *aResult.solution.as_ref().unwrap().at(0_usize).unwrap(),
        1.0
    );
    assert_eq!(
        *aResult.solution.as_ref().unwrap().at(1_usize).unwrap(),
        0.0
    );
}

#[test]
fn MathLin_LeastSquares_Test__NonRepresentableResidualIsNumericalError() {
    let mut aA: Matrix = Matrix::filled(0, 2 - 1, 0, 1 - 1, 0.0).unwrap();
    let mut aB: Vector = Vector::filled(0, 2 - 1, f64::MAX).unwrap();

    assert_eq!(
        least_squares(&aA, &aB, LeastSquaresMethod::Svd, 1e-15).status,
        Status::NumericalError
    );
    assert_eq!(
        regularized_least_squares(&aA, &aB, 1.0, 1e-15).status,
        Status::NumericalError
    );
}
