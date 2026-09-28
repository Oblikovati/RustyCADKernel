#[test]
fn MathInteg_DoubleExp_NewTest__SinhSinh_Infinite() {
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
fn MathInteg_DoubleExp_NewTest__DoubleExponential_Auto() {
    let mut aFunc: SineFunc = SineFunc;

    let aResult: IntegResult = double_exponential(
        &mut aFunc,
        0.0,
        std::f64::consts::PI,
        &DoubleExpConfig::default(),
    );

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 2.0, 1.0e-6);
}

#[test]
fn MathInteg_DoubleExp_NewTest__WithSingularity() {
    let mut aFunc: SingularFunc = SingularFunc;

    let aResult: IntegResult =
        tanh_sinh_with_singularity(&mut aFunc, 0.0, 1.0, 0.5, &DoubleExpConfig::default());

    assert!(aResult.is_done());
    assert!(aResult.absolute_error.is_some());
    near(aResult.value.unwrap(), 2.0 * (2.0_f64).sqrt(), 1.0e-6);
    assert!(aResult.number_of_points > 0);
    assert!(aResult.iterations > 0);
}
