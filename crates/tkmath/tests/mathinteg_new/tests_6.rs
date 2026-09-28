#[test]
fn MathInteg_NewTest__CompareWithOldAPI_Kronrod() {
    let mut anOldFunc: SineFuncOld = SineFuncOld;
    let mut aNewFunc: SineFunc = SineFunc;

    let anOldInteg: KronrodSingleIntegration = KronrodSingleIntegration::with_tolerance(
        &mut anOldFunc,
        0.0,
        std::f64::consts::PI,
        15,
        1.0e-10,
        100,
    );

    let mut aConfig: KronrodConfig = KronrodConfig::default();
    aConfig.nb_gauss_points = 7;
    aConfig.integration.tolerance = 1.0e-10;
    aConfig.adaptive = true;
    let aNewResult: IntegResult = kronrod(&mut aNewFunc, 0.0, std::f64::consts::PI, &aConfig);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        1.0e-8,
    );
    near(aNewResult.value.unwrap(), 2.0, 1.0e-8);
}

#[test]
fn MathInteg_NewTest__CompareWithOldAPI_Gauss() {
    let mut anOldFunc: PolynomialFuncOld = PolynomialFuncOld;
    let mut aNewFunc: PolynomialFunc = PolynomialFunc;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, 1.0, 15);

    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, 1.0, 15);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        1.0e-10,
    );
}

#[test]
fn MathInteg_NewTest__ErrorEstimation_Kronrod() {
    let mut aFunc: SineFunc = SineFunc;

    let aResult: IntegResult = kronrod_rule(&mut aFunc, 0.0, std::f64::consts::PI, 7);

    assert!(aResult.is_done());
    assert!(aResult.absolute_error.is_some());
    assert!(aResult.relative_error.is_some());

    assert!(aResult.absolute_error.unwrap() < 1.0e-10);
}
