#[test]
fn MathInteg_KronrodTest__UnattainableToleranceIsNotSuccess() {
    let mut aFunc: KinkFunc = KinkFunc;
    let mut aConfig: KronrodConfig = KronrodConfig::default();
    aConfig.integration.max_iterations = 1;
    aConfig.integration.tolerance = 1.0e-16;

    let mut aResult: IntegResult = kronrod(&mut aFunc, 0.0, 1.0, &aConfig);
    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.value.is_some());
    assert!(aResult.absolute_error.is_some());
}

#[test]
fn MathInteg_KronrodTest__DampedCosineOverPi() {
    let mut aFunc: DampedCosineFunc = DampedCosineFunc;
    let mut aConfig: KronrodConfig = KronrodConfig::new(1.0e-12, 100);
    let mut aResult: IntegResult = kronrod(&mut aFunc, 0.0, std::f64::consts::PI, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.value.is_some());
    near(aResult.value.unwrap(), 0.19135721634724556, 1.0e-11);
}

#[test]
fn MathInteg_DoubleExpTest__CallbackFailureAndLevelExhaustionAreReported() {
    let mut aFailingFunc: FailingFunc = FailingFunc;
    let mut aKinkFunc: KinkFunc = KinkFunc;
    let mut aConfig: DoubleExpConfig = DoubleExpConfig::default();
    aConfig.nb_levels = 1;

    let mut aFailure: IntegResult = tanh_sinh(&mut aFailingFunc, 0.0, 1.0, &aConfig);
    let mut anExhausted: IntegResult = tanh_sinh(&mut aKinkFunc, 0.0, 1.0, &aConfig);
    let mut anExpFailure: IntegResult = exp_sinh(&mut aFailingFunc, 0.0, &aConfig);
    let mut aSinhFailure: IntegResult = sinh_sinh(&mut aFailingFunc, &aConfig);
    assert_eq!(aFailure.status, Status::CallbackError);
    assert_eq!(anExpFailure.status, Status::CallbackError);
    assert_eq!(aSinhFailure.status, Status::CallbackError);
    assert_eq!(anExhausted.status, Status::MaxIterations);
    assert!(anExhausted.value.is_some());
    assert!(!(anExhausted.absolute_error.is_some()));
}
