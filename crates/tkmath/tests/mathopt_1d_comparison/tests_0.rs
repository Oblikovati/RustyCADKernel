#[test]
fn MathOpt_1D_ComparisonTest__Brent_Parabola() {
    let mut anOldFunc: ParabolaFunc = ParabolaFunc::default();
    let mut aNewFunc: ParabolaFunc = ParabolaFunc::default();

    let mut anOldSolver = BrentMinimum::with_options(THE_TOLERANCE, 100, 1e-12);
    anOldSolver.perform(&mut anOldFunc, 0.0, 3.0, 6.0);

    let aNewResult: ScalarResult = brent(&mut aNewFunc, 0.0, 6.0, &Config::default());

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
    near(aNewResult.root.unwrap(), 3.0, THE_TOLERANCE);
    near(aNewResult.value.unwrap(), 1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_ComparisonTest__Brent_ParabolaNegativeMinimum() {
    let mut anOldFunc: ParabolaFunc2 = ParabolaFunc2;
    let mut aNewFunc: ParabolaFunc2 = ParabolaFunc2;

    let mut anOldSolver = BrentMinimum::with_options(THE_TOLERANCE, 100, 1e-12);
    anOldSolver.perform(&mut anOldFunc, 0.0, 2.0, 5.0);

    let aNewResult: ScalarResult = brent(&mut aNewFunc, 0.0, 5.0, &Config::default());

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
    near(aNewResult.root.unwrap(), 2.0, THE_TOLERANCE);
    near(aNewResult.value.unwrap(), -1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_ComparisonTest__Brent_Cosine() {
    let mut anOldFunc: CosFunc = CosFunc::default();
    let mut aNewFunc: CosFunc = CosFunc::default();

    let mut anOldSolver = BrentMinimum::with_options(THE_TOLERANCE, 100, 1e-12);
    anOldSolver.perform(&mut anOldFunc, 2.0, std::f64::consts::PI, 4.0);

    let aNewResult: ScalarResult = brent(&mut aNewFunc, 2.0, 4.0, &Config::default());

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
    near(
        aNewResult.root.unwrap(),
        std::f64::consts::PI,
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), -1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_ComparisonTest__Brent_QuarticRight() {
    let mut anOldFunc: QuarticFunc = QuarticFunc::default();
    let mut aNewFunc: QuarticFunc = QuarticFunc::default();

    let mut anOldSolver = BrentMinimum::with_options(THE_TOLERANCE, 100, 1e-12);
    anOldSolver.perform(&mut anOldFunc, 0.0, 1.0, 2.0);

    let aNewResult: ScalarResult = brent(&mut aNewFunc, 0.0, 2.0, &Config::default());

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
    near(aNewResult.root.unwrap(), 1.0, THE_TOLERANCE);
    near(aNewResult.value.unwrap(), -1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_ComparisonTest__Brent_QuarticLeft() {
    let mut anOldFunc: QuarticFunc = QuarticFunc::default();
    let mut aNewFunc: QuarticFunc = QuarticFunc::default();

    let mut anOldSolver = BrentMinimum::with_options(THE_TOLERANCE, 100, 1e-12);
    anOldSolver.perform(&mut anOldFunc, -2.0, -1.0, 0.0);

    let aNewResult: ScalarResult = brent(&mut aNewFunc, -2.0, 0.0, &Config::default());

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
    near(aNewResult.root.unwrap(), -1.0, THE_TOLERANCE);
    near(aNewResult.value.unwrap(), -1.0, THE_TOLERANCE);
}
