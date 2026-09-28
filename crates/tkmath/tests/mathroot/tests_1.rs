#[test]
fn MathRoot_NewtonBoundedTest__SqrtTwoWithBounds() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let aResult: ScalarResult = newton_bounded(&mut aFunc, 1.5, 1.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (2.0_f64).sqrt(), THE_TOLERANCE);
}

#[test]
fn MathRoot_NewtonBoundedTest__SinWithBounds() {
    let mut aFunc: SinFunc = SinFunc::default();
    let aResult: ScalarResult = newton_bounded(&mut aFunc, 3.0, 2.0, 4.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), std::f64::consts::PI, THE_TOLERANCE);
}

#[test]
fn MathRoot_SecantTest__SqrtTwo() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let aResult: ScalarResult = secant(&mut aFunc, 1.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (2.0_f64).sqrt(), THE_TOLERANCE);
}

#[test]
fn MathRoot_SecantTest__CosMinusX() {
    let mut aFunc: CosMinusXFunc = CosMinusXFunc::default();
    let aResult: ScalarResult = secant(&mut aFunc, 0.0, 1.0, &Config::default());
    assert!(aResult.is_done());
    let aFx: f64 = (aResult.root.unwrap()).cos() - aResult.root.unwrap();
    near(aFx, 0.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_SecantTest__ExpFunction() {
    let mut aFunc: ExpMinusThreeFunc = ExpMinusThreeFunc::default();
    let aResult: ScalarResult = secant(&mut aFunc, 0.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (3.0_f64).ln(), THE_TOLERANCE);
}

#[test]
fn MathRoot_BrentTest__SqrtTwo() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 1.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (2.0_f64).sqrt(), THE_TOLERANCE);
}

#[test]
fn MathRoot_BrentTest__CosMinusX() {
    let mut aFunc: CosMinusXFunc = CosMinusXFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 1.0, &Config::default());
    assert!(aResult.is_done());
    let aFx: f64 = (aResult.root.unwrap()).cos() - aResult.root.unwrap();
    near(aFx, 0.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_BrentTest__CosMinusXHighAccuracy() {
    let mut aFunc: CosMinusXFunc = CosMinusXFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-14;
    aConfig.f_tolerance = 1.0e-14;
    let aResult: ScalarResult = brent(&mut aFunc, 0.0, 1.0, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.root.is_some());
    near(aResult.root.unwrap(), 0.73908513321516045, 1.0e-13);
}
