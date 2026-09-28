#[test]
fn MathRoot_NewtonTest__SqrtTwo() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let aResult: ScalarResult = newton(&mut aFunc, 1.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (2.0_f64).sqrt(), THE_TOLERANCE);
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_NewtonTest__CosMinusX() {
    let mut aFunc: CosMinusXFunc = CosMinusXFunc::default();
    let aResult: ScalarResult = newton(&mut aFunc, 0.5, &Config::default());
    assert!(aResult.is_done());

    let aFx: f64 = (aResult.root.unwrap()).cos() - aResult.root.unwrap();
    near(aFx, 0.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_NewtonTest__CubicEquation() {
    let mut aFunc: CubicFunc = CubicFunc::default();
    let aResult: ScalarResult = newton(&mut aFunc, 1.5, &Config::default());
    assert!(aResult.is_done());

    let aFx: f64 = aResult.root.unwrap() * aResult.root.unwrap() * aResult.root.unwrap()
        - aResult.root.unwrap()
        - 2.0;
    near(aFx, 0.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_NewtonTest__LinearFunction() {
    let mut aFunc: LinearFunc = LinearFunc::default();
    let aResult: ScalarResult = newton(&mut aFunc, 0.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), 2.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_NewtonTest__ExpFunction() {
    let mut aFunc: ExpMinusThreeFunc = ExpMinusThreeFunc::default();
    let aResult: ScalarResult = newton(&mut aFunc, 1.0, &Config::default());
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (3.0_f64).ln(), THE_TOLERANCE);
}

#[test]
fn MathRoot_NewtonTest__CustomTolerance() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-14;
    aConfig.f_tolerance = 1.0e-14;
    aConfig.max_iterations = 100;

    let aResult: ScalarResult = newton(&mut aFunc, 1.0, &aConfig);
    assert!(aResult.is_done());
    near(aResult.root.unwrap(), (2.0_f64).sqrt(), 1.0e-14);
}

#[test]
fn MathRoot_NewtonTest__RationalCosineEquation() {
    let mut aFunc: RationalCosFunc = RationalCosFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-14;
    aConfig.f_tolerance = 1.0e-14;

    let aResult: ScalarResult = newton(&mut aFunc, 1.0, &aConfig);
    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.root.is_some());
    assert!(aResult.value.is_some());
    near(aResult.root.unwrap(), 1.1887722838868333, 2.0e-13);
    near(aResult.value.unwrap(), 0.0, 1.0e-13);
}

#[test]
fn MathRoot_NewtonTest__ScaledExponentialEquation() {
    let mut aFunc: ScaledExpFunc = ScaledExpFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-14;
    aConfig.f_tolerance = 1.0e-13;

    let aResult: ScalarResult = newton(&mut aFunc, 3.5, &aConfig);
    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.root.is_some());
    assert!(aResult.value.is_some());
    near(aResult.root.unwrap(), 3.3856301402900502, 2.0e-13);
    near(aResult.value.unwrap(), 0.0, 1.0e-12);
}
