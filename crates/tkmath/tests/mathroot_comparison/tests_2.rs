#[test]
fn MathRoot_ComparisonTest__Brent_ExpMinusThree() {
    let mut anOldFunc: ExpMinusThreeFunc = ExpMinusThreeFunc::default();
    let mut aNewFunc: ExpMinusThreeFunc = ExpMinusThreeFunc::default();

    let anOldSolver =
        BracketedRoot::with_options(&mut anOldFunc, 0.0, 2.0, THE_TOLERANCE, 100, 1e-12);

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = THE_TOLERANCE;
    let aNewResult: ScalarResult = brent(&mut aNewFunc, 0.0, 2.0, &aConfig);

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
fn MathRoot_ComparisonTest__FunctionRoot_SqrtTwo() {
    let mut anOldFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let mut aNewFunc: SqrtTwoFunc = SqrtTwoFunc::default();

    let anOldSolver =
        FunctionRoot::bounded(&mut anOldFunc, 1.5, THE_TOLERANCE, 1.0, 2.0, 100).unwrap();

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = THE_TOLERANCE;
    let aNewResult: ScalarResult = newton_bounded(&mut aNewFunc, 1.5, 1.0, 2.0, &aConfig);

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
fn MathRoot_ComparisonTest__FunctionRoot_SinPi() {
    let mut anOldFunc: SinFunc = SinFunc::default();
    let mut aNewFunc: SinFunc = SinFunc::default();

    let anOldSolver =
        FunctionRoot::bounded(&mut anOldFunc, 3.0, THE_TOLERANCE, 2.0, 4.0, 100).unwrap();

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = THE_TOLERANCE;
    let aNewResult: ScalarResult = newton_bounded(&mut aNewFunc, 3.0, 2.0, 4.0, &aConfig);

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

#[test]
fn MathRoot_ComparisonTest__IterationCount_NewtonVsBrent() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;

    let aNewtonResult: ScalarResult = newton(&mut aFunc, 1.5, &aConfig);
    let aBrentResult: ScalarResult = brent(&mut aFunc, 1.0, 2.0, &aConfig);

    assert!(aNewtonResult.is_done());
    assert!(aBrentResult.is_done());

    assert!(aNewtonResult.iterations <= aBrentResult.iterations);

    near(
        aNewtonResult.root.unwrap(),
        aBrentResult.root.unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathRoot_ComparisonTest__ChallengingCase_CloseToZeroDerivative() {
    let mut old = |x: f64, out: &mut f64| {
        *out = x * x * x - 0.001;
        true
    };
    let mut new = |x: f64, out: &mut f64| {
        *out = x * x * x - 0.001;
        true
    };
    let anOldSolver = BracketedRoot::with_options(&mut old, 0.0, 1.0, THE_TOLERANCE, 100, 1e-12);
    let aNewResult = brent(&mut new, 0.0, 1.0, &Config::default());
    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());
    near(anOldSolver.root().unwrap(), 0.1, 1e-6);
    near(aNewResult.root.unwrap(), 0.1, 1e-6);
    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        2.0 * THE_TOLERANCE,
    );
}
