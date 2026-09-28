#[test]
fn MathRoot_CorrectnessTest__CallbackRefusalAndNonFiniteOutputHaveDistinctStatuses() {
    let mut aRefusingFunc = Combined(|_: f64, _: &mut f64, _: &mut f64| false);
    let mut aNonFiniteFunc = NonFiniteFunc;
    assert_eq!(
        brent(&mut aRefusingFunc, 0.0, 1.0, &Config::default()).status,
        Status::CallbackError
    );
    assert_eq!(
        secant(&mut aRefusingFunc, 0.0, 1.0, &Config::default()).status,
        Status::CallbackError
    );
    assert_eq!(
        bisection_newton(&mut aRefusingFunc, 0.0, 1.0, &Config::default()).status,
        Status::CallbackError
    );
    assert_eq!(
        brent(&mut aNonFiniteFunc, 0.0, 1.0, &Config::default()).status,
        Status::NumericalError
    );
    assert_eq!(
        secant(&mut aNonFiniteFunc, 0.0, 1.0, &Config::default()).status,
        Status::NumericalError
    );
    assert_eq!(
        bisection_newton(&mut aNonFiniteFunc, 0.0, 1.0, &Config::default()).status,
        Status::NumericalError
    );
}

#[test]
fn MathRoot_CorrectnessTest__MaxIterationStateIsCoherent() {
    let mut aFunc: SqrtTwoFunc = SqrtTwoFunc::default();
    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 1;
    aConfig.f_tolerance = 0.0;

    let aResult: ScalarResult = newton(&mut aFunc, 1.0, &aConfig);
    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.root.is_some());
    assert!(aResult.value.is_some());
    assert!(aResult.derivative.is_some());
    assert_eq!(
        aResult.value.unwrap(),
        aResult.root.unwrap() * aResult.root.unwrap() - 2.0
    );
    assert_eq!(aResult.derivative.unwrap(), 2.0 * aResult.root.unwrap());

    let aBracketed: ScalarResult = bisection(&mut aFunc, 1.0, 2.0, &aConfig);
    assert_eq!(aBracketed.status, Status::MaxIterations);
    assert!(aBracketed.root.is_some());
    assert!(aBracketed.value.is_some());
    assert_eq!(
        aBracketed.value.unwrap(),
        aBracketed.root.unwrap() * aBracketed.root.unwrap() - 2.0
    );

    let aBrent: ScalarResult = brent(&mut aFunc, 1.0, 2.0, &aConfig);
    assert_eq!(aBrent.status, Status::MaxIterations);
    assert_eq!(aBrent.iterations, 1);
    assert!(aBrent.root.is_some());
    assert!(aBrent.value.is_some());
    assert_eq!(
        aBrent.value.unwrap(),
        aBrent.root.unwrap() * aBrent.root.unwrap() - 2.0
    );
}

#[test]
fn MathRoot_CorrectnessTest__BoundedNewtonDoesNotAcceptClampedBoundary() {
    let mut aFunc = Combined(|x: f64, out: &mut f64, df: &mut f64| {
        *out = x + 1.0;
        *df = 1.0;
        true
    });
    let aResult = newton_bounded(&mut aFunc, 0.5, 0.0, 2.0, &Config::default());
    assert!(!aResult.is_done());
    assert!(aResult.value.is_some());
    assert!(aResult.value.unwrap().abs() > 0.5);
}
