#[test]
fn MathInteg_DoubleExp_NewTest__TanhSinh_Polynomial() {
    let mut aFunc: PolynomialFunc = PolynomialFunc;

    let aResult: IntegResult = tanh_sinh(&mut aFunc, 0.0, 1.0, &DoubleExpConfig::default());

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 1.0 / 3.0, 1.0e-6);
}

#[test]
fn MathInteg_DoubleExp_NewTest__TanhSinh_Sine() {
    let mut aFunc: SineFunc = SineFunc;

    let aResult: IntegResult = tanh_sinh(
        &mut aFunc,
        0.0,
        std::f64::consts::PI,
        &DoubleExpConfig::default(),
    );

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 2.0, 1.0e-6);
}

#[test]
fn MathInteg_DoubleExp_NewTest__TanhSinh_SqrtSingularity() {
    let mut aFunc: SqrtSingularityFunc = SqrtSingularityFunc;

    let mut aConfig: DoubleExpConfig = DoubleExpConfig::default();
    aConfig.nb_levels = 8;

    let aResult: IntegResult = tanh_sinh(&mut aFunc, 0.0, 1.0, &aConfig);

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 2.0, 1.0e-4);
}
