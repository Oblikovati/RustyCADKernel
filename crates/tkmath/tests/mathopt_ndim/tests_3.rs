#[test]
fn MathOpt_BFGSTest__BoothFunction() {
    let mut aFunc: BoothFunction = BoothFunction::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.0;
    *aStart.change_at(1).unwrap() = 0.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-10;
    aConfig.f_tolerance = 1.0e-12;

    let aResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(aResult.value.unwrap(), 0.0, 1.0e-8);
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-6,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        3.0,
        1.0e-6,
    );
}

#[test]
fn MathOpt_BFGSTest__RotatedQuadraticMinimum() {
    let mut aFunc: RotatedQuadraticFunction = RotatedQuadraticFunction::default();
    let mut aStart: Vector = Vector::filled(0, 2 - 1, 4.0).unwrap();
    let mut aConfig: Config = Config::default();
    aConfig.x_tolerance = 1.0e-12;
    aConfig.f_tolerance = 1.0e-12;
    let aResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.5,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        -1.0,
        1.0e-10,
    );
    near(aResult.value.unwrap(), -0.75, 1.0e-11);
}

#[test]
fn MathOpt_BFGSTest__RosenbrockFunction() {
    let mut aFunc: MathOptTest_RosenbrockFunction = MathOptTest_RosenbrockFunction::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = -1.0;
    *aStart.change_at(1).unwrap() = 1.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 500;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;

    let aResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(aResult.value.unwrap(), 0.0, 1.0e-4);
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-3,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        1.0e-3,
    );
}

#[test]
fn MathOpt_BFGSTest__BealeFunction() {
    let mut aFunc: BealeFunction = BealeFunction::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.0;
    *aStart.change_at(1).unwrap() = 0.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 200;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;

    let aResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(aResult.value.unwrap(), 0.0, 1.0e-4);
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        3.0,
        1.0e-3,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.5,
        1.0e-3,
    );
}
