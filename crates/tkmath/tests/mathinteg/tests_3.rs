#[test]
fn MathInteg_GaussTest__Order7() {
    let mut aFunc: QuadraticFunc = QuadraticFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, 1.0, 7);
    assert!(aResult.is_done());
    assert_eq!(aResult.number_of_points, 7);
    near(aResult.value.unwrap(), 1.0 / 3.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussTest__Order15() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 15);
    assert!(aResult.is_done());
    assert_eq!(aResult.number_of_points, 15);
    near(aResult.value.unwrap(), 2.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussTest__Order21() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 21);
    assert!(aResult.is_done());
    assert_eq!(aResult.number_of_points, 21);
    near(aResult.value.unwrap(), 2.0, THE_TOLERANCE);
}
