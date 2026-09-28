#[test]
fn MathOpt_NewtonTest__CompareWithOldAPI_Booth() {
    let mut anOldFunc: BoothFuncOld = BoothFuncOld::default();
    let mut aNewFunc: BoothFunc = BoothFunc::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldStart[1] = 0.0;
    anOldStart[2] = 0.0;

    let mut aNewStart: Vector = Vector::with_size(2);
    *aNewStart.change_at(0).unwrap() = 0.0;
    *aNewStart.change_at(1).unwrap() = 0.0;

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

#[test]
fn MathOpt_NewtonTest__CompareWithBFGS() {
    let mut aQuadFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aBFGSFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aBFGSConfig: Config = Config::default();
    aBFGSConfig.max_iterations = 100;
    aBFGSConfig.x_tolerance = 1.0e-10;
    aBFGSConfig.f_tolerance = 1.0e-12;

    let mut aNewtonConfig: NewtonConfig = NewtonConfig::default();
    aNewtonConfig.base.max_iterations = 50;
    aNewtonConfig.base.tolerance = 1.0e-10;

    let aNewtonResult: VectorResult = newton(&mut aQuadFunc, &aStart, &aNewtonConfig);
    let aBFGSResult: VectorResult = bfgs(&mut aBFGSFunc, &aStart, &aBFGSConfig);

    assert!(aNewtonResult.is_done());
    assert!(aBFGSResult.is_done());

    near(
        aNewtonResult.value.unwrap(),
        aBFGSResult.value.unwrap(),
        THE_TOLERANCE,
    );

    assert!(aNewtonResult.iterations <= aBFGSResult.iterations);
}

#[test]
fn MathOpt_NewtonTest__QuadraticConvergence() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 100.0;
    *aStart.change_at(1).unwrap() = 100.0;

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 10;
    aConfig.base.tolerance = 1.0e-12;

    let aResult: VectorResult = newton(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());

    assert!(aResult.iterations <= 2);
    near(aResult.value.unwrap(), 0.0, 1.0e-10);
}
