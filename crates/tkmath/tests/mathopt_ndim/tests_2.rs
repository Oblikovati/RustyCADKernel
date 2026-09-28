#[test]
fn MathOpt_PowellTest__BoothFunction() {
    let mut aFunc: BoothFunction = BoothFunction::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.0;
    *aStart.change_at(1).unwrap() = 0.0;

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
        1.0,
        1.0e-4,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        3.0,
        1.0e-4,
    );
}

#[test]
fn MathOpt_PowellTest__SphereFunction3D() {
    let mut aFunc: SphereFunction = SphereFunction::default();
    let mut aStart: Vector = Vector::with_size(3);
    *aStart.change_at(0).unwrap() = 3.0;
    *aStart.change_at(1).unwrap() = -2.0;
    *aStart.change_at(2).unwrap() = 4.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 200;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;

    let aResult: VectorResult = powell(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(aResult.value.unwrap(), 0.0, 1.0e-6);
    for i in 1..=3 {
        near(
            *aResult
                .solution
                .as_ref()
                .unwrap()
                .at((i - 1) as usize)
                .unwrap(),
            0.0,
            1.0e-4,
        );
    }
}

#[test]
fn MathOpt_PowellTest__MixedDirectionSaddleIsNotConverged() {
    let mut aFunc: BilinearSaddle = BilinearSaddle::default();
    let mut aStart: Vector = Vector::filled(0, 2 - 1, 0.0).unwrap();

    let aResult: VectorResult = powell(&mut aFunc, &aStart, &Config::default());

    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(!(aResult.is_done()));

    let mut aDirections: Matrix = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
    *aDirections.change_at(0, 0).unwrap() = 1.0;
    *aDirections.change_at(1, 1).unwrap() = 1.0;
    let aCustomResult: VectorResult =
        powell_with_directions(&mut aFunc, &aStart, &aDirections, &Config::default());
    assert_eq!(aCustomResult.status, Status::MaxIterations);
    assert!(!(aCustomResult.is_done()));
}

#[test]
fn MathOpt_BFGSTest__SimpleQuadratic() {
    let mut aFunc: SimpleQuadratic = SimpleQuadratic::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 5.0;

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
        0.0,
        1.0e-6,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        1.0e-6,
    );
    assert!(aResult.iterations < 20);
}
