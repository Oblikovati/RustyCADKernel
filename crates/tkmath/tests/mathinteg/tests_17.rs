#[test]
fn MathInteg_CorrectnessTest__SmallIntegralsUseRelativeTolerance() {
    #[derive(Default)]
    struct ScaledKinkFunc;
    impl Function for ScaledKinkFunc {
        fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
            let _ = (&theX, &theF);
            *theF = 1.0e-12 * (theX - 0.3141592653589793).abs();
            true
        }
    }
    #[derive(Default)]
    struct CountingScaledKinkFunc {
        NbCalls: usize,
    }
    impl Function for CountingScaledKinkFunc {
        fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
            let _ = (&theX, &theF);
            self.NbCalls += 1;
            *theF = 1.0e-12 * (theX - 0.3141592653589793).abs();
            true
        }
    }

    let mut aGaussFunc: ScaledKinkFunc = ScaledKinkFunc;
    let mut aGaussConfig: IntegConfig = IntegConfig::default();
    aGaussConfig.initial_order = 1;
    aGaussConfig.max_order = 2;
    aGaussConfig.max_iterations = 1;
    aGaussConfig.tolerance = 1.0e-10;
    let mut aGaussResult: IntegResult = gauss_adaptive(&mut aGaussFunc, 0.0, 1.0, &aGaussConfig);

    let mut aKronrodFunc: ScaledKinkFunc = ScaledKinkFunc;
    let mut aKronrodConfig: KronrodConfig = KronrodConfig::default();
    aKronrodConfig.integration.max_iterations = 1;
    aKronrodConfig.integration.tolerance = 1.0e-10;
    let mut aKronrodResult: IntegResult = kronrod(&mut aKronrodFunc, 0.0, 1.0, &aKronrodConfig);

    let mut aDoubleExpFunc: ScaledKinkFunc = ScaledKinkFunc;
    let mut aDoubleExpConfig: DoubleExpConfig = DoubleExpConfig::default();
    aDoubleExpConfig.nb_levels = 2;
    aDoubleExpConfig.integration.tolerance = 1.0e-10;
    let mut aDoubleExpResult: IntegResult =
        tanh_sinh(&mut aDoubleExpFunc, 0.0, 1.0, &aDoubleExpConfig);

    let mut aAutoFunc: CountingScaledKinkFunc = CountingScaledKinkFunc::default();
    let mut aAutoResult: IntegResult = kronrod_auto(&mut aAutoFunc, 0.0, 1.0, 1.0e-10, 7);

    assert_eq!(aGaussResult.status, Status::MaxIterations);
    assert_eq!(aKronrodResult.status, Status::MaxIterations);
    assert_eq!(aDoubleExpResult.status, Status::MaxIterations);
    assert_eq!(aAutoResult.number_of_points, aAutoFunc.NbCalls);
    assert!(aAutoResult.number_of_points > 15);
}

#[test]
fn MathInteg_KronrodTest__NearZeroResultHasNoRelativeErrorEstimate() {
    let mut aFunc: SinFunc = SinFunc;
    let mut aResult: IntegResult =
        kronrod_rule(&mut aFunc, -std::f64::consts::PI, std::f64::consts::PI, 7);

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
    assert!(!(aResult.relative_error.is_some()));
}

#[test]
fn MathInteg_SetTest__RejectsNegativeEquationCountBeforeAllocation() {
    #[derive(Default)]
    struct NegativeSizeFunc;
    impl FunctionSet for NegativeSizeFunc {
        fn variables(&self) -> i32 {
            1
        }
        fn equations(&self) -> i32 {
            -1
        }
        fn value(&mut self, _point: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
            let _ = &theF;
            true
        }
    }

    let mut aFunc: NegativeSizeFunc = NegativeSizeFunc;
    let mut aResult: SetResult = gauss_set(&mut aFunc, 0.0, 1.0, 3);
    assert_eq!(aResult.status, Status::InvalidInput);
    assert_eq!(aResult.nb_points, 0);
}
