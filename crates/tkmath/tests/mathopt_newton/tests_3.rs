#[test]
fn MathOpt_NewtonStatusTest__LineSearchCallbackErrorIsPreserved() {
    let mut aStart: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();

    let mut aNewtonFunc: NewtonLineSearchCallbackFailure =
        NewtonLineSearchCallbackFailure::default();
    assert_eq!(
        newton(&mut aNewtonFunc, &aStart, &NewtonConfig::default()).status,
        Status::CallbackError
    );

    let mut aLower: Vector = Vector::filled(0, 1 - 1, -2.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 2.0).unwrap();
    let mut aBoundedFunc: NewtonLineSearchCallbackFailure =
        NewtonLineSearchCallbackFailure::default();
    assert_eq!(
        newton_bounded(
            &mut aBoundedFunc,
            &aStart,
            &aLower,
            &anUpper,
            &NewtonConfig::default()
        )
        .status,
        Status::CallbackError
    );
}

#[test]
fn MathOpt_NewtonStatusTest__HessianCallbackFailureIsCallbackError() {
    let mut aFunc: HessianCallbackFailure = HessianCallbackFailure::default();
    let mut aStart: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();

    assert_eq!(
        newton(&mut aFunc, &aStart, &NewtonConfig::default()).status,
        Status::CallbackError
    );
}

#[test]
fn MathOpt_NewtonTest__CompareWithOldAPI_Quadratic() {
    let mut anOldFunc: QuadraticFuncOld = QuadraticFuncOld::default();
    let mut aNewFunc: QuadraticFunc = QuadraticFunc::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldStart[1] = 5.0;
    anOldStart[2] = 7.0;

    let mut aNewStart: Vector = Vector::with_size(2);
    *aNewStart.change_at(0).unwrap() = 5.0;
    *aNewStart.change_at(1).unwrap() = 7.0;

    let mut anOldSolver =
        NewtonMinimum::with_options(&anOldFunc, 1.0e-8, 50, 1.0e-10, true).unwrap();
    anOldSolver.perform(&mut anOldFunc, &anOldStart).unwrap();

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 50;
    aConfig.base.tolerance = 1.0e-10;
    let aNewResult: VectorResult = newton(&mut aNewFunc, &aNewStart, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        *anOldSolver.location().unwrap().at(0).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        THE_TOLERANCE,
    );
    near(
        *anOldSolver.location().unwrap().at(1).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        THE_TOLERANCE,
    );
    near(
        anOldSolver.minimum().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
}
