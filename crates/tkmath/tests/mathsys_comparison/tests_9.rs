#[test]
fn MathSys_ComparisonTest__Newton_ExponentialCircleSystem() {
    let mut aFunction: ExponentialCircleFunctionSet = ExponentialCircleFunctionSet;
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.5;
    *aStart.change_at(1).unwrap() = 0.5;

    let mut aResult: SystemResult = newton_uniform(&mut aFunction, &aStart, 1.0e-12, 1.0e-12, 100);
    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        1.0e-10,
    );
    assert!(aResult.residual_norm <= 1.0e-12);
    let mut aResidual: Vector = Vector::with_size(2);
    assert!(aFunction.value(aResult.solution.as_ref().unwrap(), &mut aResidual));
    near(*aResidual.at(0).unwrap(), 0.0, 1.0e-12);
    near(*aResidual.at(1).unwrap(), 0.0, 1.0e-12);
}

#[test]
fn MathSys_ComparisonTest__NewtonBounded_TrigonometricProductSystem() {
    let mut aFunction: TrigonometricProductFunctionSet = TrigonometricProductFunctionSet;
    let mut aStart: Vector = Vector::with_size(2);
    let mut aLower: Vector = Vector::filled(0, 2 - 1, 0.3).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 2 - 1, 0.9).unwrap();
    let mut aTolerance: Vector = Vector::filled(0, 2 - 1, 1.0e-12).unwrap();
    *aStart.change_at(0).unwrap() = 0.7;
    *aStart.change_at(1).unwrap() = 0.6;

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
        0.621_510_099_994_422_2,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.646_325_355_044_723_9,
        1.0e-10,
    );
    assert!(aResult.residual_norm <= 1.0e-12);
    let mut aResidual: Vector = Vector::with_size(2);
    assert!(aFunction.value(aResult.solution.as_ref().unwrap(), &mut aResidual));
    near(*aResidual.at(0).unwrap(), 0.0, 1.0e-12);
    near(*aResidual.at(1).unwrap(), 0.0, 1.0e-12);
}
