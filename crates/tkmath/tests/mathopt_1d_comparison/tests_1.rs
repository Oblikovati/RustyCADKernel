#[test]
fn MathOpt_1D_ComparisonTest__Brent_CoshLike() {
    let mut anOldFunc: CoshLikeFunc = CoshLikeFunc::default();
    let mut aNewFunc: CoshLikeFunc = CoshLikeFunc::default();

    let mut anOldSolver = BrentMinimum::with_options(THE_TOLERANCE, 100, 1e-12);
    anOldSolver.perform(&mut anOldFunc, -5.0, 0.0, 5.0);

    let aNewResult: ScalarResult = brent(&mut aNewFunc, -5.0, 5.0, &Config::default());

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.location().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
    near(
        anOldSolver.minimum().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.root.unwrap(), 0.0, THE_TOLERANCE);
    near(aNewResult.value.unwrap(), 2.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_ComparisonTest__Brent_Square() {
    let mut anOldFunc: SquareFunc = SquareFunc::default();
    let mut aNewFunc: SquareFunc = SquareFunc::default();

    let mut anOldSolver = BrentMinimum::with_options(THE_TOLERANCE, 100, 1e-12);
    anOldSolver.perform(&mut anOldFunc, -10.0, 0.0, 10.0);

    let aNewResult: ScalarResult = brent(&mut aNewFunc, -10.0, 10.0, &Config::default());

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.location().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
    near(
        anOldSolver.minimum().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.root.unwrap(), 0.0, THE_TOLERANCE);
    near(aNewResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_ComparisonTest__IterationCount_BrentVsGolden() {
    let mut aFunc: ParabolaFunc = ParabolaFunc::default();

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 200;

    let aBrentResult: ScalarResult = brent(&mut aFunc, 0.0, 10.0, &aConfig);
    let aGoldenResult: ScalarResult = golden(&mut aFunc, 0.0, 10.0, &aConfig);

    assert!(aBrentResult.is_done());
    assert!(aGoldenResult.is_done());

    near(
        aBrentResult.root.unwrap(),
        aGoldenResult.root.unwrap(),
        THE_TOLERANCE,
    );
    near(
        aBrentResult.value.unwrap(),
        aGoldenResult.value.unwrap(),
        THE_TOLERANCE,
    );

    assert!(aBrentResult.iterations <= aGoldenResult.iterations);
}

#[test]
fn MathOpt_1D_ComparisonTest__Accuracy_BrentVsOld() {
    let mut anOldFunc: ParabolaFunc = ParabolaFunc::default();
    let mut aNewFunc: ParabolaFunc = ParabolaFunc::default();

    let aTightTol: f64 = 1.0e-12;

    let mut anOldSolver = BrentMinimum::with_options(aTightTol, 200, 1e-12);
    anOldSolver.perform(&mut anOldFunc, 0.0, 3.0, 10.0);

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = aTightTol;
    aConfig.max_iterations = 200;
    let aNewResult: ScalarResult = brent(&mut aNewFunc, 0.0, 10.0, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(anOldSolver.location().unwrap(), 3.0, 1.0e-8);
    near(aNewResult.root.unwrap(), 3.0, 1.0e-8);
}

#[test]
fn MathOpt_1D_ComparisonTest__WideInterval_Parabola() {
    let mut anOldFunc: ParabolaFunc = ParabolaFunc::default();
    let mut aNewFunc: ParabolaFunc = ParabolaFunc::default();

    let mut anOldSolver = BrentMinimum::with_options(THE_TOLERANCE, 200, 1e-12);
    anOldSolver.perform(&mut anOldFunc, -100.0, 3.0, 100.0);

    let aNewResult: ScalarResult = brent(&mut aNewFunc, -100.0, 100.0, &Config::default());

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.location().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.root.unwrap(), 3.0, THE_TOLERANCE);
}
