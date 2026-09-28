#[test]
fn MathInteg_GaussTest__ExponentialSineOnFiniteInterval() {
    let mut aFunc: ExponentialSineFunc = ExponentialSineFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, 1.3, 21);

    assert!(aResult.is_done());
    assert!(aResult.value.is_some());
    near(aResult.value.unwrap(), 2.3771445613036857, 2.0e-13);
}

#[test]
fn MathInteg_KronrodTest__ShiftedRationalOnFiniteInterval() {
    let mut aFunc: ShiftedRationalFunc = ShiftedRationalFunc;
    let mut aConfig: KronrodConfig = KronrodConfig::new(1.0e-12, 100);
    let mut aResult: IntegResult = kronrod(&mut aFunc, -0.4, 2.3, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.value.is_some());
    near(aResult.value.unwrap(), 1.3346874620709692, 2.0e-11);
}

#[test]
fn MathInteg_GaussCompositeTest__ShiftedGaussianOnFiniteInterval() {
    let mut aFunc: ShiftedGaussianFunc = ShiftedGaussianFunc;
    let mut aResult: IntegResult = gauss_composite(&mut aFunc, -1.2, 2.1, 16, 15);

    assert!(aResult.is_done());
    assert!(aResult.value.is_some());
    near(aResult.value.unwrap(), 1.1682152570688034, 2.0e-13);
}
