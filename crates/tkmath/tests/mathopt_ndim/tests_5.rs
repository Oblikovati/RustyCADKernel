#[test]
fn MathOpt_LBFGSTest__SphereFunction10D() {
    let mut aFunc: SphereFunction = SphereFunction::default();
    let mut aStart: Vector = Vector::with_size(10);
    for i in 0..aStart.length() as usize {
        *aStart.change_at(i).unwrap() = (i + 1) as f64;
    }

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-10;
    aConfig.f_tolerance = 1.0e-12;

    let aResult: VectorResult = lbfgs(&mut aFunc, &aStart, 5, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(aResult.value.unwrap(), 0.0, 1.0e-8);
    for i in 1..=10 {
        near(
            *aResult
                .solution
                .as_ref()
                .unwrap()
                .at((i - 1) as usize)
                .unwrap(),
            0.0,
            1.0e-6,
        );
    }
}

#[test]
fn MathOpt_BFGSStatusTest__LineSearchCallbackErrorIsPreserved() {
    let mut aStart: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();

    let mut aBFGSFunc: LineSearchFailureFunction = LineSearchFailureFunction::default();
    assert_eq!(
        bfgs(&mut aBFGSFunc, &aStart, &Config::default()).status,
        Status::CallbackError
    );

    let mut anLBFGSFunc: LineSearchFailureFunction = LineSearchFailureFunction::default();
    assert_eq!(
        lbfgs(&mut anLBFGSFunc, &aStart, 10, &Config::default()).status,
        Status::CallbackError
    );

    let mut aLower: Vector = Vector::filled(0, 1 - 1, -2.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 2.0).unwrap();
    let mut aBoundedFunc: LineSearchFailureFunction = LineSearchFailureFunction::default();
    assert_eq!(
        bfgs_bounded(
            &mut aBoundedFunc,
            &aStart,
            &aLower,
            &anUpper,
            &Config::default()
        )
        .status,
        Status::CallbackError
    );

    let mut aPowellFunc: LineSearchFailureFunction = LineSearchFailureFunction::default();
    assert_eq!(
        powell(&mut aPowellFunc, &aStart, &Config::default()).status,
        Status::CallbackError
    );
}

#[test]
fn MathOpt_BFGSStatusTest__LineSearchNonFiniteValueIsNumericalError() {
    let mut aFunc = LineSearchFailureFunction {
        nonfinite: true,
        calls: 0,
    };
    let mut aStart: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();

    assert_eq!(
        bfgs(&mut aFunc, &aStart, &Config::default()).status,
        Status::NumericalError
    );
}

#[test]
fn MathOpt_NDimStatusTest__DirectCallbackFailureIsCallbackError() {
    let mut aStart: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();

    let mut aBFGSFunc: GradientCallbackFailureFunction = GradientCallbackFailureFunction::default();
    assert_eq!(
        bfgs(&mut aBFGSFunc, &aStart, &Config::default()).status,
        Status::CallbackError
    );

    let mut aPowellFunc: ValueCallbackFailureFunction = ValueCallbackFailureFunction::default();
    assert_eq!(
        powell(&mut aPowellFunc, &aStart, &Config::default()).status,
        Status::CallbackError
    );
}
