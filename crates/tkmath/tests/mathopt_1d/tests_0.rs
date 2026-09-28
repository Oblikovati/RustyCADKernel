#[test]
fn MathOpt_1D_BrentTest__SimpleParabola() {
    let mut aFunc: ParabolaFunc = ParabolaFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 10.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 3.0, THE_TOLERANCE);
    near(aResult.value.unwrap(), 1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_BrentTest__ParabolaWithNegativeMinimum() {
    let mut aFunc: ParabolaFunc2 = ParabolaFunc2;
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 5.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 2.0, THE_TOLERANCE);
    near(aResult.value.unwrap(), -1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_BrentTest__CosineMinimum() {
    let mut aFunc: CosFunc = CosFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 2.0, 4.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), std::f64::consts::PI, THE_TOLERANCE);
    near(aResult.value.unwrap(), -1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_BrentTest__QuarticMinimumRight() {
    let mut aFunc: QuarticFunc = QuarticFunc::default();

    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 1.0, THE_TOLERANCE);
    near(aResult.value.unwrap(), -1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_BrentTest__QuarticMinimumLeft() {
    let mut aFunc: QuarticFunc = QuarticFunc::default();

    let aResult: ScalarResult = brent(&mut aFunc, -2.0, 0.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), -1.0, THE_TOLERANCE);
    near(aResult.value.unwrap(), -1.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_1D_BrentTest__CoshLikeMinimum() {
    let mut aFunc: CoshLikeFunc = CoshLikeFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, -5.0, 5.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 0.0, THE_TOLERANCE);
    near(aResult.value.unwrap(), 2.0, THE_TOLERANCE);
}
