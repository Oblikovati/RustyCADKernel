#[test]
fn MathInteg_AccuracyTest__HigherOrderMoreAccurate() {
    let mut aFunc: SinFunc = SinFunc;
    let aExact: f64 = 2.0;

    let mut aResult3: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 3);
    let mut aResult7: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 7);
    let mut aResult15: IntegResult = gauss(&mut aFunc, 0.0, std::f64::consts::PI, 15);

    assert!(aResult3.is_done());
    assert!(aResult7.is_done());
    assert!(aResult15.is_done());

    let aError3: f64 = (aResult3.value.unwrap() - aExact).abs();
    let aError7: f64 = (aResult7.value.unwrap() - aExact).abs();
    let aError15: f64 = (aResult15.value.unwrap() - aExact).abs();

    assert!(aError3 > aError7);
    assert!(aError7 > aError15);
}

#[test]
fn MathInteg_CorrectnessTest__FixedRulesPropagateCallbackAndNonFiniteFailures() {
    let mut aFailingFunc: FailingFunc = FailingFunc;
    let mut aNonFiniteFunc: NonFiniteFunc = NonFiniteFunc;

    let mut aGaussCallback: IntegResult = gauss(&mut aFailingFunc, 0.0, 1.0, 7);
    let mut aGaussNonFinite: IntegResult = gauss(&mut aNonFiniteFunc, 0.0, 1.0, 7);
    let mut aKronrodCallback: IntegResult = kronrod_rule(&mut aFailingFunc, 0.0, 1.0, 7);
    let mut aKronrodNonFinite: IntegResult = kronrod_rule(&mut aNonFiniteFunc, 0.0, 1.0, 7);

    assert_eq!(aGaussCallback.status, Status::CallbackError);
    assert_eq!(aGaussNonFinite.status, Status::NumericalError);
    assert_eq!(aKronrodCallback.status, Status::CallbackError);
    assert_eq!(aKronrodNonFinite.status, Status::NumericalError);
    assert_eq!(aGaussCallback.number_of_points, 1);
    assert_eq!(aGaussNonFinite.number_of_points, 1);
    assert_eq!(aKronrodCallback.number_of_points, 1);
    assert_eq!(aKronrodNonFinite.number_of_points, 1);
}

#[test]
fn MathInteg_CorrectnessTest__AdaptiveRulesPreserveChildCallbackStatus() {
    struct DelayedFailure {
        successful: usize,
        calls: usize,
    }
    impl Function for DelayedFailure {
        fn value(&mut self, x: f64, value: &mut f64) -> bool {
            let previous = self.calls;
            self.calls += 1;
            if previous >= self.successful {
                return false;
            }
            *value = (x - 0.3141592653589793).abs();
            true
        }
    }

    let mut aGaussFunc: DelayedFailure = DelayedFailure {
        successful: 3,
        calls: 0,
    };
    let mut aGaussConfig: IntegConfig = IntegConfig::default();
    aGaussConfig.initial_order = 1;
    aGaussConfig.max_order = 2;
    aGaussConfig.max_iterations = 10;
    aGaussConfig.tolerance = 1.0e-15;
    let mut aGaussResult: IntegResult = gauss_adaptive(&mut aGaussFunc, 0.0, 1.0, &aGaussConfig);

    let mut aKronrodFunc: DelayedFailure = DelayedFailure {
        successful: 15,
        calls: 0,
    };
    let mut aKronrodConfig: KronrodConfig = KronrodConfig::default();
    aKronrodConfig.integration.max_iterations = 10;
    aKronrodConfig.integration.tolerance = 1.0e-15;
    let mut aKronrodResult: IntegResult = kronrod(&mut aKronrodFunc, 0.0, 1.0, &aKronrodConfig);

    assert_eq!(aGaussResult.status, Status::CallbackError);
    assert_eq!(aGaussResult.number_of_points, 4);
    assert_eq!(aKronrodResult.status, Status::CallbackError);
    assert_eq!(aKronrodResult.number_of_points, 16);
}
