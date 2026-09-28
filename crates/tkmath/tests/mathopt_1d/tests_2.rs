#[test]
fn MathOpt_1D_GoldenTest__QuarticMinimum() {
    let mut aFunc: QuarticFunc = QuarticFunc::default();
    let aResult: ScalarResult = golden(&mut aFunc, 0.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 1.0, THE_TOLERANCE);
    near(aResult.value.unwrap(), -1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_GoldenTest__CoshLikeMinimum() {
    let mut aFunc: CoshLikeFunc = CoshLikeFunc::default();
    let aResult: ScalarResult = golden(&mut aFunc, -5.0, 5.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 0.0, THE_TOLERANCE);
    near(aResult.value.unwrap(), 2.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_BrentWithBracketTest__SimpleParabola() {
    let mut aFunc: ParabolaFunc = ParabolaFunc::default();
    let aResult: ScalarResult = brent_with_bracket(&mut aFunc, 0.0, 1.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 3.0, THE_TOLERANCE);
    near(aResult.value.unwrap(), 1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_BrentWithBracketTest__StartNearMinimum() {
    let mut aFunc: ParabolaFunc = ParabolaFunc::default();
    let aResult: ScalarResult = brent_with_bracket(&mut aFunc, 2.5, 0.5, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 3.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_BrentWithBracketTest__CoshLike() {
    let mut aFunc: CoshLikeFunc = CoshLikeFunc::default();

    let aResult: ScalarResult = brent_with_bracket(&mut aFunc, 0.1, 0.2, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_BrentWithBracketTest__BracketFailureStatusIsPreserved() {
    let mut aCallbackFailure = |_: f64, _: &mut f64| false;

    assert_eq!(
        brent_with_bracket(&mut aCallbackFailure, 0.0, 1.0, &Config::default()).status,
        Status::CallbackError
    );

    let mut aMonotonicFunction = |x: f64, f: &mut f64| {
        *f = -x;
        true
    };

    assert_eq!(
        brent_with_bracket(&mut aMonotonicFunction, 0.0, 1.0, &Config::default()).status,
        Status::MaxIterations
    );
}
