#[test]
fn MathOpt_NewtonTest__Sphere3D() {
    let mut aFunc: SphereFunc = SphereFunc::default();

    let mut aStart: Vector = Vector::with_size(3);
    *aStart.change_at(0).unwrap() = 3.0;
    *aStart.change_at(1).unwrap() = -2.0;
    *aStart.change_at(2).unwrap() = 4.0;

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 50;
    aConfig.base.tolerance = 1.0e-10;

    let aResult: VectorResult = newton(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    for i in 0..aResult.solution.as_ref().unwrap().length() as usize {
        near(
            *aResult.solution.as_ref().unwrap().at(i).unwrap(),
            0.0,
            THE_TOLERANCE,
        );
    }
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);

    assert!(aResult.iterations <= 5);
}

#[test]
fn MathOpt_NewtonTest__ModifiedNewton_Quadratic() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 50;
    aConfig.base.tolerance = 1.0e-10;

    let aResult: VectorResult = newton_modified(&mut aFunc, &aStart, &aConfig);

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
}

#[test]
fn MathOpt_NewtonTest__ModifiedNewton_Rosenbrock() {
    let mut aFunc: RosenbrockFunc = RosenbrockFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = -1.0;
    *aStart.change_at(1).unwrap() = 1.0;

    let mut aConfig: NewtonConfig = NewtonConfig::default();
    aConfig.base.max_iterations = 200;
    aConfig.base.tolerance = 1.0e-6;
    aConfig.regularization = 1.0e-4;

    let aResult: VectorResult = newton_modified(&mut aFunc, &aStart, &aConfig);

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
}
