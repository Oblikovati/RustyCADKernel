#[test]
fn MathInteg_GaussAdaptiveTest__UsesConfiguredOrders() {
    let mut aFunc: QuadraticFunc = QuadraticFunc;
    let mut aConfig: IntegConfig = IntegConfig::default();
    aConfig.initial_order = 9;
    aConfig.max_order = 18;
    aConfig.tolerance = 1.0e-12;
    aConfig.max_iterations = 2;

    let mut aResult: IntegResult = gauss_adaptive(&mut aFunc, 0.0, 1.0, &aConfig);
    assert!(aResult.is_done());
    assert_eq!(aResult.number_of_points, 27);
    near(aResult.value.unwrap(), 1.0 / 3.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussCompositeTest__SineFunction() {
    let mut aFunc: SinFunc = SinFunc;

    let mut aResult: IntegResult = gauss_composite(&mut aFunc, 0.0, std::f64::consts::PI, 10, 7);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 2.0, THE_TOLERANCE);
    assert_eq!(aResult.number_of_points, 70);
}

#[test]
fn MathInteg_GaussCompositeTest__OscillatoryFunction() {
    let mut aFunc: OscillatoryFunc = OscillatoryFunc;

    let mut aResult: IntegResult = gauss_composite(&mut aFunc, 0.0, std::f64::consts::PI, 50, 7);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 0.0, 1.0e-8);
}
