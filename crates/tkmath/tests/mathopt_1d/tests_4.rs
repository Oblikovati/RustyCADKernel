#[test]
fn MathOpt_1D_BoolConversionTest__SuccessfulResultIsTrue() {
    let mut aFunc: ParabolaFunc = ParabolaFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 10.0, &Config::default());
    assert!(bool::from(&aResult));
}

#[test]
fn MathOpt_1D_IterationTest__BrentConvergesQuickly() {
    let mut aFunc: ParabolaFunc = ParabolaFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 10.0, &Config::default());
    assert!(aResult.is_done());
    assert!(aResult.iterations < 50);
}

#[test]
fn MathOpt_1D_IterationTest__GoldenConverges() {
    let mut aFunc: ParabolaFunc = ParabolaFunc::default();
    let aResult: ScalarResult = golden(&mut aFunc, 0.0, 10.0, &Config::default());
    assert!(aResult.is_done());
    assert!(aResult.iterations < 100);
}
