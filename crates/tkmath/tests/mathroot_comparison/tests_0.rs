#[test]
fn MathRoot_ComparisonTest__BisectionNewton_SqrtTwo() {
    let mut anOldFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let mut aNewFunc: SqrtTwoFunc = SqrtTwoFunc::default();

    let mut anOldSolver = BissecNewton::new(THE_TOLERANCE);
    anOldSolver.perform(&mut anOldFunc, 1.0, 2.0, 100);

    let aNewResult: ScalarResult = bisection_newton(&mut aNewFunc, 1.0, 2.0, &Config::default());

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
    near(anOldSolver.root().unwrap(), (2.0_f64).sqrt(), THE_TOLERANCE);
}

#[test]
fn MathRoot_ComparisonTest__BisectionNewton_CosMinusX() {
    let mut anOldFunc: CosMinusXFunc = CosMinusXFunc::default();
    let mut aNewFunc: CosMinusXFunc = CosMinusXFunc::default();

    let mut anOldSolver = BissecNewton::new(THE_TOLERANCE);
    anOldSolver.perform(&mut anOldFunc, 0.0, 1.0, 100);

    let aNewResult: ScalarResult = bisection_newton(&mut aNewFunc, 0.0, 1.0, &Config::default());

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );

    let aFx: f64 = (aNewResult.root.unwrap()).cos() - aNewResult.root.unwrap();
    near(aFx, 0.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_ComparisonTest__BisectionNewton_SinPi() {
    let mut anOldFunc: SinFunc = SinFunc::default();
    let mut aNewFunc: SinFunc = SinFunc::default();

    let mut anOldSolver = BissecNewton::new(THE_TOLERANCE);
    anOldSolver.perform(&mut anOldFunc, 2.0, 4.0, 100);

    let aNewResult: ScalarResult = bisection_newton(&mut aNewFunc, 2.0, 4.0, &Config::default());

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
fn MathRoot_ComparisonTest__BisectionNewton_Cubic() {
    let mut anOldFunc: CubicFunc = CubicFunc::default();
    let mut aNewFunc: CubicFunc = CubicFunc::default();

    let mut anOldSolver = BissecNewton::new(THE_TOLERANCE);
    anOldSolver.perform(&mut anOldFunc, 1.0, 2.0, 100);

    let aNewResult: ScalarResult = bisection_newton(&mut aNewFunc, 1.0, 2.0, &Config::default());

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathRoot_ComparisonTest__Newton_SqrtTwo() {
    let mut anOldFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let mut aNewFunc: SqrtTwoFunc = SqrtTwoFunc::default();

    let anOldSolver =
        NewtonFunctionRoot::with_iterations(&mut anOldFunc, 1.5, THE_TOLERANCE, THE_TOLERANCE, 100);

    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = THE_TOLERANCE;
    aConfig.f_tolerance = THE_TOLERANCE;
    let aNewResult: ScalarResult = newton(&mut aNewFunc, 1.5, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldSolver.root().unwrap(),
        aNewResult.root.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.root.unwrap(), (2.0_f64).sqrt(), THE_TOLERANCE);
}
