#[test]
fn MathRoot_ComparisonTest__Newton_ExpMinusThree() {
    let mut anOldFunc: ExpMinusThreeFunc = ExpMinusThreeFunc::default();
    let mut aNewFunc: ExpMinusThreeFunc = ExpMinusThreeFunc::default();

    let anOldSolver =
        NewtonFunctionRoot::with_iterations(&mut anOldFunc, 1.0, THE_TOLERANCE, THE_TOLERANCE, 100);

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = THE_TOLERANCE;
    aConfig.f_tolerance = THE_TOLERANCE;
    let aNewResult: ScalarResult = newton(&mut aNewFunc, 1.0, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.root.unwrap(), (3.0_f64).ln(), THE_TOLERANCE);
}

#[test]
fn MathRoot_ComparisonTest__Newton_CosMinusX() {
    let mut anOldFunc: CosMinusXFunc = CosMinusXFunc::default();
    let mut aNewFunc: CosMinusXFunc = CosMinusXFunc::default();

    let anOldSolver =
        NewtonFunctionRoot::with_iterations(&mut anOldFunc, 0.5, THE_TOLERANCE, THE_TOLERANCE, 100);

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = THE_TOLERANCE;
    aConfig.f_tolerance = THE_TOLERANCE;
    let aNewResult: ScalarResult = newton(&mut aNewFunc, 0.5, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathRoot_ComparisonTest__Brent_SqrtTwo() {
    let mut anOldFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let mut aNewFunc: SqrtTwoFunc = SqrtTwoFunc::default();

    let anOldSolver =
        BracketedRoot::with_options(&mut anOldFunc, 1.0, 2.0, THE_TOLERANCE, 100, 1e-12);

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = THE_TOLERANCE;
    let aNewResult: ScalarResult = brent(&mut aNewFunc, 1.0, 2.0, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.root.unwrap(), (2.0_f64).sqrt(), THE_TOLERANCE);
}

#[test]
fn MathRoot_ComparisonTest__Brent_CosMinusX() {
    let mut anOldFunc: CosMinusXFunc = CosMinusXFunc::default();
    let mut aNewFunc: CosMinusXFunc = CosMinusXFunc::default();

    let anOldSolver =
        BracketedRoot::with_options(&mut anOldFunc, 0.0, 1.0, THE_TOLERANCE, 100, 1e-12);

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = THE_TOLERANCE;
    let aNewResult: ScalarResult = brent(&mut aNewFunc, 0.0, 1.0, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathRoot_ComparisonTest__Brent_SinPi() {
    let mut anOldFunc: SinFunc = SinFunc::default();
    let mut aNewFunc: SinFunc = SinFunc::default();

    let anOldSolver =
        BracketedRoot::with_options(&mut anOldFunc, 2.0, 4.0, THE_TOLERANCE, 100, 1e-12);

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = THE_TOLERANCE;
    let aNewResult: ScalarResult = brent(&mut aNewFunc, 2.0, 4.0, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
    near(
        aNewResult.root.unwrap(),
        std::f64::consts::PI,
        THE_TOLERANCE,
    );
}
