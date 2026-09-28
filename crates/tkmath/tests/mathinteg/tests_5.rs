#[test]
fn MathInteg_GaussTest__PolynomialExactness_Order3() {
    let mut aFunc: PolynomialFunc = PolynomialFunc;

    let mut aResult: IntegResult = gauss(&mut aFunc, -1.0, 1.0, 3);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussTest__PolynomialExactness_Order7() {
    let mut aFunc: PolynomialFunc = PolynomialFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, -1.0, 1.0, 7);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussAdaptiveTest__SineFunction() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aConfig: IntegConfig = IntegConfig::default();
    aConfig.tolerance = 1.0e-12;

    let mut aResult: IntegResult = gauss_adaptive(&mut aFunc, 0.0, std::f64::consts::PI, &aConfig);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 2.0, 1.0e-11);
}
