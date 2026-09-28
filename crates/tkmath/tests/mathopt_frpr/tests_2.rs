#[test]
fn MathOpt_FRPRTest__NumericalGradient_Quadratic() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aConfig: FrprConfig = FrprConfig::default();
    aConfig.base.max_iterations = 100;
    aConfig.base.tolerance = 1.0e-6;

    let aResult: VectorResult = frpr_numerical(&mut aFunc, &aStart, 1.0e-8, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-4,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        1.0e-4,
    );
    near(aResult.value.unwrap(), 0.0, 1.0e-4);
}

#[test]
fn MathOpt_FRPRStatusTest__LineSearchEvaluationStatusIsPreserved() {
    let start = Vector::filled(0, 0, 1.0).unwrap();
    let mut f = LineFailure {
        nonfinite: false,
        calls: 0,
    };
    assert_eq!(
        frpr(&mut f, &start, &FrprConfig::default()).status,
        Status::CallbackError
    );
    let mut f = LineFailure {
        nonfinite: true,
        calls: 0,
    };
    assert_eq!(
        frpr(&mut f, &start, &FrprConfig::default()).status,
        Status::NumericalError
    );
}

#[test]
fn MathOpt_FRPRTest__CompareWithOldAPI_Quadratic() {
    let mut anOldFunc: QuadraticFuncOld = QuadraticFuncOld;
    let mut aNewFunc: QuadraticFunc = QuadraticFunc::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldStart[1] = 5.0;
    anOldStart[2] = 7.0;

    let mut aNewStart: Vector = Vector::with_size(2);
    *aNewStart.change_at(0).unwrap() = 5.0;
    *aNewStart.change_at(1).unwrap() = 7.0;

    let mut anOldSolver = Frpr::with_options(&anOldFunc, 1.0e-8, 100, 1e-12).unwrap();
    anOldSolver.perform(&mut anOldFunc, &anOldStart).unwrap();

    let mut aConfig: FrprConfig = FrprConfig::default();
    aConfig.base.max_iterations = 100;
    aConfig.base.tolerance = 1.0e-8;
    let aNewResult: VectorResult = frpr(&mut aNewFunc, &aNewStart, &aConfig);

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
