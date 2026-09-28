#[test]
fn MathInteg_SpecialTest__SqrtFunction() {
    let mut aFunc: SqrtFunc = SqrtFunc;

    let mut aResult: IntegResult = gauss_composite(&mut aFunc, 0.001, 1.0, 20, 15);
    assert!(aResult.is_done());

    let aExpected: f64 = (2.0 / 3.0) * (1.0 - (0.001_f64).powf(1.5));
    near(aResult.value.unwrap(), aExpected, 1.0e-6);
}

#[test]
fn MathInteg_BoolConversionTest__SuccessfulResultIsTrue() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 15);
    assert!(bool::from(&aResult));
}

#[test]
fn MathInteg_BoolConversionTest__InvalidInputIsFalse() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 0);
    assert!(!(bool::from(&aResult)));
}
