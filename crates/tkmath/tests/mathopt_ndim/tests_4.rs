#[test]
fn MathOpt_BFGSTest__SphereFunction5D() {
    let mut aFunc: SphereFunction = SphereFunction::default();
    let mut aStart: Vector = Vector::with_size(5);
    for i in 0..aStart.length() as usize {
        *aStart.change_at(i).unwrap() = (i + 1) as f64;
    }

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-10;
    aConfig.f_tolerance = 1.0e-12;

    let aResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(aResult.value.unwrap(), 0.0, 1.0e-8);
    for i in 1..=5 {
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
    assert!(aResult.iterations < 20);
}

#[test]
fn MathOpt_BFGSNumericalTest__SimpleQuadratic() {
    let mut aFunc: SimpleQuadratic = SimpleQuadratic::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 5.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;

    let aResult: VectorResult = bfgs_numerical(&mut aFunc, &aStart, 1.0e-7, &aConfig);

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

#[test]
fn MathOpt_LBFGSTest__SimpleQuadratic() {
    let mut aFunc: SimpleQuadratic = SimpleQuadratic::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 5.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-10;
    aConfig.f_tolerance = 1.0e-12;

    let aResult: VectorResult = lbfgs(&mut aFunc, &aStart, 10, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(aResult.value.unwrap(), 0.0, 1.0e-8);
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        1.0e-6,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        1.0e-6,
    );
}

#[test]
fn MathOpt_LBFGSTest__BoothFunction() {
    let mut aFunc: BoothFunction = BoothFunction::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.0;
    *aStart.change_at(1).unwrap() = 0.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-10;
    aConfig.f_tolerance = 1.0e-12;

    let aResult: VectorResult = lbfgs(&mut aFunc, &aStart, 10, &aConfig);

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
