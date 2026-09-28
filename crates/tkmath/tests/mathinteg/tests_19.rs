#[test]
fn MathInteg_GaussCompositeTest__RationalOscillationOnFiniteInterval() {
    let mut aFunc: RationalOscillationFunc = RationalOscillationFunc;
    let mut aResult: IntegResult = gauss_composite(&mut aFunc, 0.2, 3.4, 64, 15);

    assert!(aResult.is_done());
    assert!(aResult.value.is_some());
    near(aResult.value.unwrap(), -0.023002601281013002, 2.0e-12);
}

#[test]
fn MathInteg_KronrodTest__LogCosineOnAsymmetricInterval() {
    let mut aFunc: LogCosineFunc = LogCosineFunc;
    let mut aConfig: KronrodConfig = KronrodConfig::new(1.0e-12, 100);
    let mut aResult: IntegResult = kronrod(&mut aFunc, -0.7, 2.4, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.value.is_some());
    near(aResult.value.unwrap(), 1.4805767078645715, 2.0e-11);
}

#[test]
fn MathInteg_DoubleExpTest__ReversedSquareRootLogInterval() {
    let mut aFunc: SquareRootLogFunc = SquareRootLogFunc;
    let mut aConfig: DoubleExpConfig = DoubleExpConfig::new(1.0e-11, 100);
    aConfig.nb_levels = 12;
    let mut aResult: IntegResult = tanh_sinh(&mut aFunc, 2.0, 0.0, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.value.is_some());
    near(aResult.value.unwrap(), -1.426_347_068_120_945, 2.0e-9);
}
