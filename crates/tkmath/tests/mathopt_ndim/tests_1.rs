#[test]
fn MathOpt_PowellTest__SeparableQuarticNonzeroMinimum() {
    let mut aFunc: MathOptTest_SeparableQuarticFunction =
        MathOptTest_SeparableQuarticFunction::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = -1.5;
    *aStart.change_at(1).unwrap() = 1.5;
    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 200;
    aConfig.x_tolerance = 1.0e-10;
    aConfig.f_tolerance = 1.0e-12;

    let aResult: VectorResult = powell(&mut aFunc, &aStart, &aConfig);

    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.5673642266809229,
        2.0e-6,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        -0.6823278038280193,
        2.0e-6,
    );
    near(aResult.value.unwrap(), 4.254626565881520, 1.0e-9);
}

#[test]
fn MathOpt_BFGSTest__ExponentialQuadraticConvexMinimum() {
    let mut aFunc: MathOptTest_ExponentialQuadraticFunction =
        MathOptTest_ExponentialQuadraticFunction::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = -2.0;
    *aStart.change_at(1).unwrap() = 2.0;
    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 150;
    aConfig.x_tolerance = 1.0e-11;
    aConfig.f_tolerance = 1.0e-11;

    let aResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.6872331928700078,
        1.0e-7,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        -1.156383403564996,
        1.0e-7,
    );
    near(aResult.value.unwrap(), 0.7722682277234190, 1.0e-10);
}

#[test]
fn MathOpt_BFGSBoundedTest__CoupledQuadraticActiveUpperBoundary() {
    let mut aFunc: MathOptTest_ActiveBoundaryFunction =
        MathOptTest_ActiveBoundaryFunction::default();
    let mut aStart: Vector = Vector::filled(0, 2 - 1, 0.5).unwrap();
    let mut aLower: Vector = Vector::with_size(2);
    let mut anUpper: Vector = Vector::with_size(2);
    *aLower.change_at(0).unwrap() = -1.0;
    *aLower.change_at(1).unwrap() = 0.0;
    *anUpper.change_at(0).unwrap() = 1.0;
    *anUpper.change_at(1).unwrap() = 2.0;
    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-11;
    aConfig.f_tolerance = 1.0e-11;

    let aResult: VectorResult = bfgs_bounded(&mut aFunc, &aStart, &aLower, &anUpper, &aConfig);

    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.111111111111111,
        1.0e-7,
    );
    near(aResult.value.unwrap(), 1.138888888888889, 1.0e-10);
}

#[test]
fn MathOpt_PowellTest__SimpleQuadratic() {
    let mut aFunc: SimpleQuadratic = SimpleQuadratic::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 5.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 200;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;

    let aResult: VectorResult = powell(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(aResult.value.unwrap(), 0.0, 1.0e-6);
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        1.0e-4,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        1.0e-4,
    );
}
