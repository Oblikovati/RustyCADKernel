#[test]
fn MathSys_ComparisonTest__NewtonBounded_UsesFeasibleGradientForActiveSet() {
    let mut aSystem: CoupledBoundedSystem = CoupledBoundedSystem;
    let mut aStart: Vector = Vector::filled(0, 2 - 1, 0.0).unwrap();
    let mut aLower: Vector = Vector::with_size(2);
    let mut anUpper: Vector = Vector::with_size(2);
    *aLower.change_at(0).unwrap() = 0.0;
    *aLower.change_at(1).unwrap() = -2.0;
    *anUpper.change_at(0).unwrap() = 2.0;
    *anUpper.change_at(1).unwrap() = 0.0;
    let mut aTolerance: Vector = Vector::filled(0, 2 - 1, 1.0e-12).unwrap();

    let mut aResult: SystemResult = newton_bounded(
        &mut aSystem,
        &aStart,
        &aLower,
        &anUpper,
        &aTolerance,
        1.0e-12,
        10,
    );
    assert!(aResult.solution.is_some());
    assert!(*aResult.solution.as_ref().unwrap().at(0).unwrap() > 0.1);
    assert!(aResult.residual_norm < 1.0);
}

#[test]
fn MathSys_ComparisonTest__Linearized_UnequalColumnsRetainMinimumNormSolution() {
    let mut aMatrix: Matrix = Matrix::filled(0, 1 - 1, 0, 2 - 1, 0.0).unwrap();
    *aMatrix.change_at(0, 0).unwrap() = 1.0;
    *aMatrix.change_at(0, 1).unwrap() = 100.0;
    let mut aRhs: Vector = Vector::filled(1, 1, 1.0).unwrap();

    let aStep: linear::LinearStep =
        linear::solve_linearized(&aMatrix, &aRhs, linear::RANK_TOLERANCE);
    assert!(aStep.solution.is_some());
    assert_eq!(aStep.rank, 1);
    near(
        *aStep.solution.as_ref().unwrap().at(0).unwrap(),
        1.0 / 10001.0,
        1.0e-10,
    );
    near(
        *aStep.solution.as_ref().unwrap().at(1).unwrap(),
        100.0 / 10001.0,
        1.0e-10,
    );
}
