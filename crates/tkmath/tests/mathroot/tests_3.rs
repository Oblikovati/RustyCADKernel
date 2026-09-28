#[test]
fn MathRoot_BrentTest__InverseQuadraticInterpolationConvergesOnAsymmetricCubic() {
    let mut aFunc: CubicFunc = CubicFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-14;
    aConfig.f_tolerance = 0.0;
    aConfig.max_iterations = 100;

    let aResult: ScalarResult = brent(&mut aFunc, 1.0, 2.0, &aConfig);
    assert!(aResult.is_done());
    assert!(aResult.root.is_some());
    assert!(aResult.iterations < 12);
    near(aResult.root.unwrap(), 1.5213797068045676, 1.0e-14);
}

#[test]
fn MathRoot_BrentTest__ZeroFunctionToleranceConvergesPositionally() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-14;
    aConfig.f_tolerance = 0.0;
    aConfig.max_iterations = 100;

    let aResult: ScalarResult = brent(&mut aFunc, 1.0, 2.0, &aConfig);
    assert!(aResult.is_done());
    assert!(aResult.root.is_some());
    near(aResult.root.unwrap(), (2.0_f64).sqrt(), 2.0e-14);
}

#[test]
fn MathRoot_BrentTest__CallbackRefusalDuringIterationIsPropagated() {
    let mut aFunc = FailingFunc::default();
    let aResult = brent(&mut aFunc, 1.0, 2.0, &Config::default());
    assert_eq!(aResult.status, Status::CallbackError);
    assert_eq!(aResult.iterations, 1);
    assert!(aResult.root.is_none());
    assert!(aResult.value.is_none());
}

#[test]
fn MathRoot_BisectionTest__SqrtTwo() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let aResult: ScalarResult = bisection(&mut aFunc, 1.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (2.0_f64).sqrt(), THE_TOLERANCE);
}

#[test]
fn MathRoot_BisectionTest__CosMinusX() {
    let mut aFunc: CosMinusXFunc = CosMinusXFunc::default();
    let aResult: ScalarResult = bisection(&mut aFunc, 0.0, 1.0, &Config::default());
    assert!(aResult.is_done());
    let aFx: f64 = (aResult.root.unwrap()).cos() - aResult.root.unwrap();
    near(aFx, 0.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_BisectionTest__SinPi() {
    let mut aFunc: SinFunc = SinFunc::default();
    let aResult: ScalarResult = bisection(&mut aFunc, 2.0, 4.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), std::f64::consts::PI, THE_TOLERANCE);
}

#[test]
fn MathRoot_BisectionTest__InvalidBracket() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();

    let aResult: ScalarResult = bisection(&mut aFunc, 2.0, 3.0, &Config::default());
    assert_eq!(aResult.status, Status::InvalidInput);
}

#[test]
fn MathRoot_BisectionNewtonTest__SqrtTwo() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let aResult: ScalarResult = bisection_newton(&mut aFunc, 1.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (2.0_f64).sqrt(), THE_TOLERANCE);
}
