#[test]
fn MathOpt_1D_BrentTest__CallbackFailureIsCallbackError() {
    let mut aFunc = |_: f64, _: &mut f64| false;

    assert_eq!(
        brent(&mut aFunc, 0.0, 1.0, &Config::default()).status,
        Status::CallbackError
    );
}

#[test]
fn MathOpt_1D_ComparisonTest__BrentVsGoldenSameResult() {
    let mut aFunc: ParabolaFunc = ParabolaFunc::default();

    let aBrent: ScalarResult = brent(&mut aFunc, 0.0, 10.0, &Config::default());
    let aGolden: ScalarResult = golden(&mut aFunc, 0.0, 10.0, &Config::default());

    assert!(aBrent.is_done());
    assert!(aGolden.is_done());

    near(aBrent.root.unwrap(), aGolden.root.unwrap(), THE_TOLERANCE);
    near(aBrent.value.unwrap(), aGolden.value.unwrap(), THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_ComparisonTest__BrentFasterThanGolden() {
    let mut aFunc: ParabolaFunc = ParabolaFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 200;

    let aBrent: ScalarResult = brent(&mut aFunc, 0.0, 10.0, &aConfig);
    let aGolden: ScalarResult = golden(&mut aFunc, 0.0, 10.0, &aConfig);

    assert!(aBrent.is_done());
    assert!(aGolden.is_done());

    assert!(aBrent.iterations <= aGolden.iterations);
}

#[test]
fn MathOpt_1D_RobustnessTest__MinimumAtLeftBoundary() {
    let mut aFunc = |x: f64, f: &mut f64| {
        *f = x * x;
        true
    };
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 10.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_RobustnessTest__MinimumAtRightBoundary() {
    let mut aFunc = |x: f64, f: &mut f64| {
        *f = (10.0 - x) * (10.0 - x);
        true
    };
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 10.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 10.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_RobustnessTest__VeryFlatFunction() {
    let mut aFunc = |x: f64, f: &mut f64| {
        *f = {
            let d: f64 = x - 5.0;
            d * d * d * d
        };
        true
    };
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 10.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 5.0, 1.0e-5);
}
