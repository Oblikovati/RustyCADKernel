#[test]
fn MathInteg_GaussCompositeTest__GaussianFunction() {
    let mut aFunc: GaussianFunc = GaussianFunc;

    let mut aResult: IntegResult = gauss_composite(&mut aFunc, -3.0, 3.0, 50, 15);
    assert!(aResult.is_done());

    near(
        aResult.value.unwrap(),
        (std::f64::consts::PI).sqrt(),
        1.0e-4,
    );
}

#[test]
fn MathInteg_GaussCompositeTest__InvalidSubintervals() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult = gauss_composite(&mut aFunc, 0.0, std::f64::consts::PI, 0, 7);
    assert_eq!(aResult.status, Status::InvalidInput);
}

#[test]
fn MathInteg_BoundsTest__ReversedBounds() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, std::f64::consts::PI, 0.0, 15);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), -2.0, THE_TOLERANCE);
}
