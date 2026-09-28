#[test]
fn MathInteg_NewTest__ErrorEstimation_TanhSinh() {
    let mut aFunc: SineFunc = SineFunc;

    let aResult: IntegResult = tanh_sinh(
        &mut aFunc,
        0.0,
        std::f64::consts::PI,
        &DoubleExpConfig::default(),
    );

    assert!(aResult.is_done());
}

#[test]
fn MathInteg_Kronrod_NewTest__InfiniteInterval() {
    let mut aFunc: InfiniteGaussianFunc = InfiniteGaussianFunc;

    let aResult: IntegResult = sinh_sinh(&mut aFunc, &DoubleExpConfig::default());

    assert!(aResult.is_done());
    near(
        aResult.value.unwrap(),
        (std::f64::consts::PI).sqrt(),
        1.0e-4,
    );
}

#[test]
fn MathInteg_Kronrod_NewTest__SemiInfiniteInterval() {
    let mut aFunc: ExponentialDecayFunc = ExponentialDecayFunc;

    let aResult: IntegResult = exp_sinh(&mut aFunc, 0.0, &DoubleExpConfig::default());

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 1.0, 1.0e-4);
}
