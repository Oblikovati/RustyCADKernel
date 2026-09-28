#[test]
fn MathInteg_GaussTest__ReciprocalFunction() {
    let mut aFunc: ReciprocalFunc = ReciprocalFunc;

    let mut aResult: IntegResult = gauss(&mut aFunc, 1.0, (1.0_f64).exp(), 15);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 1.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussTest__Order3() {
    let mut aFunc: QuadraticFunc = QuadraticFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, 1.0, 3);
    assert!(aResult.is_done());
    assert_eq!(aResult.number_of_points, 3);
    near(aResult.value.unwrap(), 1.0 / 3.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_GaussTest__Order5() {
    let mut aFunc: QuadraticFunc = QuadraticFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, 1.0, 5);
    assert!(aResult.is_done());
    assert_eq!(aResult.number_of_points, 5);
    near(aResult.value.unwrap(), 1.0 / 3.0, THE_TOLERANCE);
}
