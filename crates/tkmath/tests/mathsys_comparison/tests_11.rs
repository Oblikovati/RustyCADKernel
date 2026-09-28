#[test]
fn MathSys_ComparisonTest__Newton_OverdeterminedTranscendentalSystem() {
    let mut aFunction: OverdeterminedTranscendentalFunctionSet =
        OverdeterminedTranscendentalFunctionSet;
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.4;
    *aStart.change_at(1).unwrap() = 0.9;

    let mut aResult: SystemResult = newton_uniform(&mut aFunction, &aStart, 1.0e-12, 1.0e-12, 100);
    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    assert_eq!(aResult.rank, 2);
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.6,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.8,
        1.0e-10,
    );
    assert!(aResult.residual_norm <= 1.0e-12);
    let mut aResidual: Vector = Vector::with_size(3);
    assert!(aFunction.value(aResult.solution.as_ref().unwrap(), &mut aResidual));
    for anIndex in 0..aResidual.length() as usize {
        near(*aResidual.at(anIndex).unwrap(), 0.0, 1.0e-12);
    }
}

#[test]
fn MathSys_ComparisonTest__NewtonBounded_LogarithmicSystem() {
    let mut aFunction: BoundedLogarithmicFunctionSet = BoundedLogarithmicFunctionSet;
    let mut aStart: Vector = Vector::with_size(2);
    let mut aLower: Vector = Vector::with_size(2);
    let mut anUpper: Vector = Vector::with_size(2);
    let mut aTolerance: Vector = Vector::filled(0, 2 - 1, 1.0e-12).unwrap();
    *aStart.change_at(0).unwrap() = 0.4;
    *aStart.change_at(1).unwrap() = 1.2;
    *aLower.change_at(0).unwrap() = 0.01;
    *aLower.change_at(1).unwrap() = 0.8;
    *anUpper.change_at(0).unwrap() = 0.8;
    *anUpper.change_at(1).unwrap() = 1.4;

    let mut aResult: SystemResult = newton_bounded(
        &mut aFunction,
        &aStart,
        &aLower,
        &anUpper,
        &aTolerance,
        1.0e-12,
        100,
    );
    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.055810767307769064,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.2023689235833408,
        1.0e-10,
    );
    assert!(aResult.residual_norm <= 1.0e-12);
    let mut aResidual: Vector = Vector::with_size(2);
    assert!(aFunction.value(aResult.solution.as_ref().unwrap(), &mut aResidual));
    near(*aResidual.at(0).unwrap(), 0.0, 1.0e-12);
    near(*aResidual.at(1).unwrap(), 0.0, 1.0e-12);
}
