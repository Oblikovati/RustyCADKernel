#[test]
fn MathOpt_NewtonTest__Quadratic() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 50;
    aConfig.base.tolerance = 1.0e-10;

    let aResult: VectorResult = newton(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        THE_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);

    assert!(aResult.iterations <= 5);
}

#[test]
fn MathOpt_NewtonTest__Booth() {
    let mut aFunc: BoothFunc = BoothFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.0;
    *aStart.change_at(1).unwrap() = 0.0;

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 50;
    aConfig.base.tolerance = 1.0e-10;

    let aResult: VectorResult = newton(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        3.0,
        THE_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_NewtonTest__Rosenbrock() {
    let mut aFunc: RosenbrockFunc = RosenbrockFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.5;
    *aStart.change_at(1).unwrap() = 0.5;

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 100;
    aConfig.base.tolerance = 1.0e-8;

    let aResult: VectorResult = newton(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
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
    near(aResult.value.unwrap(), 0.0, 1.0e-4);
}
