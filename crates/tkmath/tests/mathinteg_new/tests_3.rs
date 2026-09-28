#[test]
fn MathInteg_DoubleExp_NewTest__TanhSinh_LogSingularity() {
    let mut aFunc: LogSingularityFunc = LogSingularityFunc;

    let mut aConfig: DoubleExpConfig = DoubleExpConfig::default();
    aConfig.nb_levels = 8;

    let aResult: IntegResult = tanh_sinh(&mut aFunc, 0.0, 1.0, &aConfig);

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 1.0, 1.0e-4);
}

#[test]
fn MathInteg_DoubleExp_NewTest__TanhSinhSingular() {
    let mut aFunc: SqrtSingularityFunc = SqrtSingularityFunc;

    let aResult: IntegResult = tanh_sinh_singular(&mut aFunc, 0.0, 1.0, 1.0e-6);

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 2.0, 1.0e-3);
}

#[test]
fn MathInteg_DoubleExp_NewTest__ExpSinh_SemiInfinite() {
    let mut aFunc: ExponentialDecayFunc = ExponentialDecayFunc;

    let aResult: IntegResult = exp_sinh(&mut aFunc, 0.0, &DoubleExpConfig::default());

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 1.0, 1.0e-4);
}
