#[test]
fn MathInteg_CorrectnessTest__FiniteRulesHandleZeroWidthAndRejectNonFiniteIntervals() {
    let mut aFunc: FailingFunc = FailingFunc;
    let anInf: f64 = f64::INFINITY;

    for aResult in [
        gauss(&mut aFunc, 1.0, 1.0, 100),
        gauss_adaptive(&mut aFunc, 1.0, 1.0, &IntegConfig::default()),
        gauss_composite(&mut aFunc, 1.0, 1.0, 4, 100),
        kronrod_rule(&mut aFunc, 1.0, 1.0, 7),
        kronrod(&mut aFunc, 1.0, 1.0, &KronrodConfig::default()),
        kronrod_auto(&mut aFunc, 1.0, 1.0, 1e-10, 30),
        tanh_sinh(&mut aFunc, 1.0, 1.0, &DoubleExpConfig::default()),
        double_exponential(&mut aFunc, 1.0, 1.0, &DoubleExpConfig::default()),
    ] {
        assert!(aResult.is_done());
        assert!(aResult.value.is_some());
        assert_eq!(aResult.value.unwrap(), 0.0);
        assert_eq!(aResult.number_of_points, 0);
    }

    assert_eq!(
        gauss(&mut aFunc, 0.0, anInf, 7).status,
        Status::InvalidInput
    );
    assert_eq!(
        kronrod_rule(&mut aFunc, 0.0, 1.0, 31).status,
        Status::InvalidInput
    );
    assert_eq!(
        tanh_sinh(&mut aFunc, 0.0, anInf, &DoubleExpConfig::default()).status,
        Status::InvalidInput
    );
}

#[test]
fn MathInteg_GaussTest__OrderAboveLegacyLimitIsClamped() {
    let mut aFunc: ConstantFunc = ConstantFunc;
    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, 1.0, 100);
    assert!(aResult.is_done());
    assert!(aResult.value.is_some());
    near(aResult.value.unwrap(), 5.0, THE_TOLERANCE);
    assert_eq!(aResult.number_of_points, 61);
}

#[test]
fn MathInteg_GaussAdaptiveTest__WorkBudgetExhaustionRetainsEstimateAndError() {
    let mut aFunc: KinkFunc = KinkFunc;
    let mut aConfig: IntegConfig = IntegConfig::default();
    aConfig.initial_order = 1;
    aConfig.max_order = 2;
    aConfig.max_iterations = 1;
    aConfig.tolerance = 1.0e-15;

    let mut aResult: IntegResult = gauss_adaptive(&mut aFunc, 0.0, 1.0, &aConfig);
    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.value.is_some());
    assert!(aResult.absolute_error.is_some());
    assert!(aResult.absolute_error.unwrap() > 0.0);
    assert_eq!(aResult.iterations, 1);
    assert_eq!(aResult.number_of_points, 3);
}
