#[test]
fn MathInteg_GaussTest__SineFunction() {
    let mut aFunc: SinFunc = SinFunc;

    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 15);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 2.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussTest__CosineFunction() {
    let mut aFunc: CosFunc = CosFunc;

    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI / 2.0, 15);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 1.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussTest__ExponentialFunction() {
    let mut aFunc: ExpFunc = ExpFunc;

    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, 1.0, 15);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), (1.0_f64).exp() - 1.0, THE_TOLERANCE);
}
