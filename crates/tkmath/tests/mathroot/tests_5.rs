#[test]
fn MathRoot_BoolConversionTest__InvalidInputIsFalse() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let aResult: ScalarResult = brent(&mut aFunc, 2.0, 3.0, &Config::default());
    assert!(!(bool::from(&aResult)));
}

#[test]
fn MathRoot_CorrectnessTest__BracketedMethodsAcceptExactEndpoints() {
    let mut aFunc: EndpointFunc = EndpointFunc::default();
    for aResult in [
        bisection(&mut aFunc, 1.0, 3.0, &Config::default()),
        bisection_newton(&mut aFunc, 1.0, 3.0, &Config::default()),
        brent(&mut aFunc, 1.0, 3.0, &Config::default()),
        newton_bounded(&mut aFunc, 2.0, 1.0, 3.0, &Config::default()),
        bisection(&mut aFunc, 0.0, 1.0, &Config::default()),
        bisection_newton(&mut aFunc, 0.0, 1.0, &Config::default()),
        brent(&mut aFunc, 0.0, 1.0, &Config::default()),
        newton_bounded(&mut aFunc, 0.5, 0.0, 1.0, &Config::default()),
    ] {
        assert!(aResult.is_done());
        assert!(aResult.root.is_some());
        assert!(aResult.value.is_some());
        assert_eq!(aResult.root.unwrap(), 1.0);
        assert_eq!(aResult.value.unwrap(), 0.0);
    }
}

#[test]
fn MathRoot_CorrectnessTest__NewtonBoundedNormalizesReversedFiniteBounds() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    assert_eq!(
        bisection(&mut aFunc, 2.0, 1.0, &Config::default()).status,
        Status::InvalidInput
    );
    assert_eq!(
        bisection_newton(&mut aFunc, 2.0, 1.0, &Config::default()).status,
        Status::InvalidInput
    );
    assert_eq!(
        brent(&mut aFunc, 2.0, 1.0, &Config::default()).status,
        Status::InvalidInput
    );
    let aNewton: ScalarResult = newton_bounded(&mut aFunc, 1.5, 2.0, 1.0, &Config::default());
    assert!(aNewton.is_done());
    assert!(aNewton.root.is_some());
    near(aNewton.root.unwrap(), (2.0_f64).sqrt(), 1.0e-10);
    assert_eq!(
        brent(&mut aFunc, 1.0, f64::INFINITY, &Config::default()).status,
        Status::InvalidInput
    );
}

#[test]
fn MathRoot_CorrectnessTest__InvalidConfigurationIsRejected() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 0;
    assert_eq!(
        newton(&mut aFunc, 1.0, &aConfig).status,
        Status::InvalidInput
    );
    aConfig.max_iterations = 10;
    aConfig.f_tolerance = f64::NAN;
    assert_eq!(
        secant(&mut aFunc, 1.0, 2.0, &aConfig).status,
        Status::InvalidInput
    );
}

#[test]
fn MathRoot_CorrectnessTest__NonFiniteCallbackOutputIsNumericalError() {
    let mut aFunc: NonFiniteFunc = NonFiniteFunc::default();
    assert_eq!(
        bisection(&mut aFunc, 0.0, 2.0, &Config::default()).status,
        Status::NumericalError
    );
    assert_eq!(
        newton(&mut aFunc, 1.0, &Config::default()).status,
        Status::NumericalError
    );
}

#[test]
fn MathRoot_CorrectnessTest__CallbackFailureMidSolveIsPropagated() {
    let mut aBisectionFunc: FailingFunc = FailingFunc::default();
    let mut aNewtonFunc: FailingFunc = FailingFunc::default();
    assert_eq!(
        bisection(&mut aBisectionFunc, 1.0, 2.0, &Config::default()).status,
        Status::CallbackError
    );
    assert_eq!(
        newton(&mut aNewtonFunc, 1.0, &Config::default()).status,
        Status::CallbackError
    );
}

#[test]
fn MathRoot_CorrectnessTest__BracketConvergenceDoesNotRequireExactIrrationalResidual() {
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-12;
    aConfig.f_tolerance = 0.0;
    aConfig.max_iterations = 100;

    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    for aResult in [
        bisection(&mut aFunc, 1.0, 2.0, &aConfig),
        bisection_newton(&mut aFunc, 1.0, 2.0, &aConfig),
        brent(&mut aFunc, 1.0, 2.0, &aConfig),
    ] {
        assert!(aResult.is_done());
        assert!(aResult.root.is_some());
        near(
            aResult.root.unwrap(),
            (2.0_f64).sqrt(),
            2.0 * aConfig.x_tolerance,
        );
    }
}

#[test]
fn MathRoot_CorrectnessTest__NewtonTinyStepStillRequiresAcceptableResidual() {
    let mut aFunc = Combined(|_: f64, out: &mut f64, df: &mut f64| {
        *out = 1.0;
        *df = 1e300;
        true
    });
    let aConfig = Config {
        x_tolerance: 1e-10,
        f_tolerance: 0.0,
        max_iterations: 2,
        ..Default::default()
    };
    assert!(!newton(&mut aFunc, 1.0, &aConfig).is_done());
}
