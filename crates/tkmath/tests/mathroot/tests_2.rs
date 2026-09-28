#[test]
fn MathRoot_BrentTest__CubicEquation() {
    let mut aFunc: CubicFunc = CubicFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 1.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    let aFx: f64 = aResult.root.unwrap() * aResult.root.unwrap() * aResult.root.unwrap()
        - aResult.root.unwrap()
        - 2.0;
    near(aFx, 0.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_BrentTest__SinPi() {
    let mut aFunc: SinFunc = SinFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 2.0, 4.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), std::f64::consts::PI, THE_TOLERANCE);
}

#[test]
fn MathRoot_BrentTest__ExpFunction() {
    let mut aFunc: ExpMinusThreeFunc = ExpMinusThreeFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (3.0_f64).ln(), THE_TOLERANCE);
}

#[test]
fn MathRoot_BrentTest__ExponentialLevel() {
    let mut aFunc: ExpMinusFiftyFunc = ExpMinusFiftyFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-14;
    aConfig.f_tolerance = 1.0e-13;

    let aResult: ScalarResult = brent(&mut aFunc, 3.0, 5.0, &aConfig);
    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.root.is_some());
    assert!(aResult.value.is_some());
    near(aResult.root.unwrap(), 3.9120230054281464, 2.0e-13);
    near(aResult.value.unwrap(), 0.0, 1.0e-12);
}

#[test]
fn MathRoot_BrentTest__ShiftedSineUpperBranch() {
    let mut aFunc: ShiftedSinFunc = ShiftedSinFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-14;
    aConfig.f_tolerance = 1.0e-14;

    let aResult: ScalarResult = brent(&mut aFunc, 2.0, 4.0, &aConfig);
    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.root.is_some());
    assert!(aResult.value.is_some());
    near(aResult.root.unwrap(), 2.8889123984477143, 2.0e-13);
    near(aResult.value.unwrap(), 0.0, 1.0e-13);
}

#[test]
fn MathRoot_BrentTest__InvalidBracket() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();

    let aResult: ScalarResult = brent(&mut aFunc, 2.0, 3.0, &Config::default());
    assert_eq!(aResult.status, Status::InvalidInput);
}

#[test]
fn MathRoot_BrentTest__ReversedBracket() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 2.0, 1.0, &Config::default());
    assert_eq!(aResult.status, Status::InvalidInput);
}

#[test]
fn MathRoot_BrentTest__LargeMagnitudeBracketUsesFloatingPointScaleTolerance() {
    let aLower: f64 = 1e16;
    let aUpper: f64 = aLower.next_up();
    let mut aFunc = |x: f64, out: &mut f64| {
        *out = if x < aUpper { -1.0 } else { 1.0 };
        true
    };
    let aConfig = Config {
        x_tolerance: 1e-20,
        f_tolerance: 0.0,
        ..Default::default()
    };
    let aResult = brent(&mut aFunc, aLower, aUpper, &aConfig);
    assert!(aResult.is_done());
    assert!(aResult.root.is_some());
    assert!(aResult.root.unwrap() == aLower || aResult.root.unwrap() == aUpper);
}
