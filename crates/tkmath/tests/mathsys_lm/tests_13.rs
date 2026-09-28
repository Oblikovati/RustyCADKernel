#[test]
fn MathSys_LM_Test__LinearizedSolveUsesRankTolerance() {
    let mut aMatrix: Matrix = Matrix::filled(0, 2 - 1, 0, 2 - 1, 0.0).unwrap();
    *aMatrix.change_at(0, 0).unwrap() = 1.0;
    *aMatrix.change_at(0, 1).unwrap() = 1.0;
    *aMatrix.change_at(1, 1).unwrap() = 1.0e-8;
    let mut aRhs: Vector = Vector::filled(1, 2, 0.0).unwrap();
    *aRhs.change_at(0).unwrap() = 1.0;
    *aRhs.change_at(1).unwrap() = 0.0;

    let aStep: linear::LinearStep = linear::solve_linearized(&aMatrix, &aRhs, 1.0e-6);
    assert!(aStep.solution.is_some());
    assert_eq!(aStep.rank, 1);
}

#[test]
fn MathSys_LM_Test__CallbackFalseIsCallbackError() {
    let mut aFunc: CallbackFailure = CallbackFailure;
    let mut aStart: Vector = Vector::filled(1, 1, 0.0).unwrap();
    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &LMConfig::default());
    assert_eq!(aResult.status, Status::CallbackError);
}
