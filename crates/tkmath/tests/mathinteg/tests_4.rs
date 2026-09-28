#[test]
fn MathInteg_GaussTest__Order9() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 9);
    assert!(aResult.is_done());
    assert_eq!(aResult.number_of_points, 9);
    near(aResult.value.unwrap(), 2.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussTest__Order61() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 61);
    assert!(aResult.is_done());
    assert_eq!(aResult.number_of_points, 61);
    near(aResult.value.unwrap(), 2.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussTest__InvalidOrderNonPositive() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 0);
    assert_eq!(aResult.status, Status::InvalidInput);
}
