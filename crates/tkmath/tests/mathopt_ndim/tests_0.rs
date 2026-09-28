#[test]
fn MathOpt_BFGSTest__RotatedQuadratic2DGeneralMinimum() {
    let mut aFunc: MathOptTest_RotatedQuadratic2DFunction =
        MathOptTest_RotatedQuadratic2DFunction::default();
    let mut aStart: Vector = Vector::filled(0, 2 - 1, 4.0).unwrap();
    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-12;
    aConfig.f_tolerance = 1.0e-12;

    let aResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        2.333333333333333,
        1.0e-9,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        -2.833333333333333,
        1.0e-9,
    );
    near(aResult.value.unwrap(), -9.833333333333332, 1.0e-10);
}

#[test]
fn MathOpt_BFGSTest__RotatedQuadratic3DLeastSquares() {
    let mut aFunc: MathOptTest_RotatedQuadratic3DFunction =
        MathOptTest_RotatedQuadratic3DFunction::default();
    let mut aStart: Vector = Vector::with_size(3);
    *aStart.change_at(0).unwrap() = -3.0;
    *aStart.change_at(1).unwrap() = 4.0;
    *aStart.change_at(2).unwrap() = 2.0;
    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-12;
    aConfig.f_tolerance = 1.0e-12;

    let aResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        2.0,
        1.0e-9,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        -1.0,
        1.0e-9,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(2).unwrap(),
        -1.0,
        1.0e-9,
    );
    near(aResult.value.unwrap(), 0.75, 1.0e-10);
}

#[test]
fn MathOpt_BFGSTest__LargeObjectiveOffsetDoesNotImplyConvergence() {
    let mut aFunc: MathOptTest_LargeOffsetQuadraticFunction =
        MathOptTest_LargeOffsetQuadraticFunction::default();
    let mut aStart: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 3.0;
    aConfig.f_tolerance = 1.0e-12;
    aConfig.relative_tolerance = 0.0;

    let aResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert_eq!(aResult.status, Status::NotConverged);
    assert!(aResult.gradient.is_some());
    assert!((*aResult.gradient.as_ref().unwrap().at(0).unwrap()).abs() > aConfig.f_tolerance);
}

#[test]
fn MathOpt_LBFGSTest__WeightedShiftedSphere4D() {
    let mut aFunc: MathOptTest_WeightedShiftedSphereFunction =
        MathOptTest_WeightedShiftedSphereFunction::default();
    let mut aStart: Vector = Vector::with_size(4);
    *aStart.change_at(0).unwrap() = -4.0;
    *aStart.change_at(1).unwrap() = 3.0;
    *aStart.change_at(2).unwrap() = -2.0;
    *aStart.change_at(3).unwrap() = 5.0;
    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 150;
    aConfig.x_tolerance = 1.0e-11;
    aConfig.f_tolerance = 1.0e-11;

    let aResult: VectorResult = lbfgs(&mut aFunc, &aStart, 6, &aConfig);

    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.25,
        1.0e-7,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        -2.5,
        1.0e-7,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(2).unwrap(),
        0.75,
        1.0e-7,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(3).unwrap(),
        -1.0,
        1.0e-7,
    );
    near(aResult.value.unwrap(), 4.5, 1.0e-10);
}
