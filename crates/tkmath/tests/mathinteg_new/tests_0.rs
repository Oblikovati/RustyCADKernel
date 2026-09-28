#[test]
fn MathInteg_Kronrod_NewTest__KronrodRule_Polynomial() {
    let mut aFunc: PolynomialFunc = PolynomialFunc;

    let aResult: IntegResult = kronrod_rule(&mut aFunc, 0.0, 1.0, 7);

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 1.0 / 3.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_Kronrod_NewTest__KronrodRule_Sine() {
    let mut aFunc: SineFunc = SineFunc;

    let aResult: IntegResult = kronrod_rule(&mut aFunc, 0.0, std::f64::consts::PI, 7);

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 2.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_Kronrod_NewTest__AdaptiveKronrod_Oscillatory() {
    let mut aFunc: OscillatoryFunc = OscillatoryFunc;

    let mut aConfig: KronrodConfig = KronrodConfig::default();
    aConfig.integration.tolerance = 1.0e-10;
    aConfig.nb_gauss_points = 7;
    aConfig.adaptive = true;
    aConfig.integration.max_iterations = 100;

    let aResult: IntegResult = kronrod(&mut aFunc, 0.0, std::f64::consts::PI, &aConfig);

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 0.0, 1.0e-6);
}
