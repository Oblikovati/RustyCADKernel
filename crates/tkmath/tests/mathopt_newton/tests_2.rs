#[test]
fn MathOpt_NewtonTest__NumericalHessian_Quadratic() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 50;
    aConfig.base.tolerance = 1.0e-8;

    let aResult: VectorResult = newton_numerical_hessian(&mut aFunc, &aStart, 1.0e-6, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-4,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        1.0e-4,
    );
    near(aResult.value.unwrap(), 0.0, 1.0e-6);
}

#[test]
fn MathOpt_NewtonTest__FullNumerical_Quadratic() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 50;
    aConfig.base.tolerance = 1.0e-6;

    let aResult: VectorResult = newton_numerical(&mut aFunc, &aStart, 1.0e-8, 1.0e-6, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-3,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        1.0e-3,
    );
    near(aResult.value.unwrap(), 0.0, 1.0e-4);
}

#[test]
fn MathOpt_NewtonTest__BoundedCallbacksStayInsideBounds() {
    let mut aStart: Vector = Vector::filled(0, 2 - 1, 0.5).unwrap();
    let mut aLower: Vector = Vector::filled(0, 2 - 1, 0.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 2 - 1, 1.0).unwrap();

    let mut aNewtonFunc: BoundsCheckingQuadratic = BoundsCheckingQuadratic::default();
    let mut aNewtonConfig: NewtonConfig = NewtonConfig::default();
    aNewtonConfig.base.max_iterations = 10;
    let aNewtonResult: VectorResult =
        newton_bounded(&mut aNewtonFunc, &aStart, &aLower, &anUpper, &aNewtonConfig);

    assert!(aNewtonResult.is_done());
    assert!(aNewtonResult.solution.is_some());
    assert!(aNewtonResult.value.is_some());
    near(
        *aNewtonResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        1.0e-10,
    );
    near(
        *aNewtonResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        1.0e-10,
    );
    near(aNewtonResult.value.unwrap(), 0.0, 1.0e-12);
    assert_eq!(aNewtonFunc.NbOutsideCalls, 0);

    let mut aBFGSFunc: BoundsCheckingQuadratic = BoundsCheckingQuadratic::default();
    let mut aBFGSConfig: Config = Config::default();
    aBFGSConfig.max_iterations = 10;
    let aBFGSResult: VectorResult =
        bfgs_bounded(&mut aBFGSFunc, &aStart, &aLower, &anUpper, &aBFGSConfig);

    assert!(aBFGSResult.is_done());
    assert!(aBFGSResult.solution.is_some());
    assert!(aBFGSResult.value.is_some());
    near(
        *aBFGSResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        1.0e-10,
    );
    near(
        *aBFGSResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        1.0e-10,
    );
    near(aBFGSResult.value.unwrap(), 0.0, 1.0e-12);
    assert_eq!(aBFGSFunc.NbOutsideCalls, 0);
}
