#[test]
fn MathInteg_Kronrod_NewTest__AdaptiveKronrod_Gaussian() {
    let mut aFunc: GaussianFunc = GaussianFunc;

    let mut aConfig: DoubleExpConfig = DoubleExpConfig::default();
    aConfig.integration.tolerance = 1.0e-10;

    let aResult: IntegResult = sinh_sinh(&mut aFunc, &aConfig);

    assert!(aResult.is_done());
    near(
        aResult.value.unwrap(),
        (std::f64::consts::PI).sqrt(),
        1.0e-6,
    );
}

#[test]
fn MathInteg_Kronrod_NewTest__KronrodAuto() {
    let mut aFunc: SineFunc = SineFunc;

    let aResult: IntegResult = kronrod_auto(&mut aFunc, 0.0, 2.0 * std::f64::consts::PI, 1e-10, 30);

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 0.0, 1.0e-8);
}

#[test]
fn MathInteg_Kronrod_NewTest__DifferentOrders() {
    let mut aFunc: PolynomialFunc = PolynomialFunc;

    let aOrders: [usize; 5] = [3, 5, 7, 10, 15];

    for aOrder in aOrders {
        let aResult: IntegResult = kronrod_rule(&mut aFunc, 0.0, 1.0, aOrder);

        assert!(aResult.is_done());
        near(aResult.value.unwrap(), 1.0 / 3.0, 1.0e-8);
    }
}
