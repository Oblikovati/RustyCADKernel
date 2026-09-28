#[test]
fn MathInteg_GaussAdaptiveTest__ExponentialFunction() {
    let mut aFunc: ExpFunc = ExpFunc;
    let mut aConfig: IntegConfig = IntegConfig::default();
    aConfig.tolerance = 1.0e-12;

    let mut aResult: IntegResult = gauss_adaptive(&mut aFunc, 0.0, 1.0, &aConfig);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), (1.0_f64).exp() - 1.0, 1.0e-11);
}

#[test]
fn MathInteg_GaussAdaptiveTest__OscillatoryFunction() {
    let mut aFunc: OscillatoryFunc = OscillatoryFunc;
    let mut aConfig: IntegConfig = IntegConfig::default();
    aConfig.tolerance = 1.0e-8;
    aConfig.max_iterations = 20;

    let mut aResult: IntegResult = gauss_adaptive(&mut aFunc, 0.0, std::f64::consts::PI, &aConfig);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 0.0, 1.0e-6);
}

#[test]
fn MathInteg_GaussAdaptiveTest__ProvidesErrorEstimate() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aConfig: IntegConfig = IntegConfig::default();
    aConfig.tolerance = 1.0e-8;

    let mut aResult: IntegResult = gauss_adaptive(&mut aFunc, 0.0, std::f64::consts::PI, &aConfig);
    assert!(aResult.is_done());
    assert!(aResult.absolute_error.is_some());
    assert!(aResult.relative_error.is_some());
    assert!((aResult.absolute_error.unwrap()).is_finite());
    assert!((aResult.relative_error.unwrap()).is_finite());
    assert!(aResult.absolute_error.unwrap() >= 0.0);
    assert!(aResult.relative_error.unwrap() >= 0.0);
    assert!(aResult.absolute_error.unwrap() < 1.0e-6);
}
