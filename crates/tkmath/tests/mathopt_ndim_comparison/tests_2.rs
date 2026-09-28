#[test]
fn MathOpt_NDim_ComparisonTest__BFGS_Rosenbrock() {
    let mut anOldFunc: RosenbrockFuncGradOld = RosenbrockFuncGradOld::default();
    let mut aNewFunc: RosenbrockFuncNew = RosenbrockFuncNew::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldStart[1] = -1.0;
    anOldStart[2] = 1.0;

    let mut aNewStart: Vector = Vector::with_size(2);
    *aNewStart.change_at(0).unwrap() = -1.0;
    *aNewStart.change_at(1).unwrap() = 1.0;

    let mut anOldSolver = Bfgs::with_options(2, 1.0e-6, 1000, 1e-12).unwrap();
    anOldSolver.perform(&mut anOldFunc, &anOldStart).unwrap();

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 1000;
    aConfig.x_tolerance = 1.0e-6;
    aConfig.f_tolerance = 1.0e-8;
    let aNewResult: VectorResult = bfgs(&mut aNewFunc, &aNewStart, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        *anOldSolver.location().unwrap().at(0).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0e-3,
    );
    near(
        *anOldSolver.location().unwrap().at(1).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0e-3,
    );

    near(
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-3,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        1.0e-3,
    );
}

#[test]
fn MathOpt_NDim_ComparisonTest__BFGS_Sphere3D() {
    let mut anOldFunc: SphereFuncGradOld = SphereFuncGradOld { myN: 3 };
    let mut aNewFunc: SphereFuncNew = SphereFuncNew::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 3).unwrap();
    anOldStart[1] = 3.0;
    anOldStart[2] = -2.0;
    anOldStart[3] = 4.0;

    let mut aNewStart: Vector = Vector::with_size(3);
    *aNewStart.change_at(0).unwrap() = 3.0;
    *aNewStart.change_at(1).unwrap() = -2.0;
    *aNewStart.change_at(2).unwrap() = 4.0;

    let mut anOldSolver = Bfgs::with_options(3, 1.0e-8, 100, 1e-12).unwrap();
    anOldSolver.perform(&mut anOldFunc, &anOldStart).unwrap();

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;
    let aNewResult: VectorResult = bfgs(&mut aNewFunc, &aNewStart, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *anOldSolver.location().unwrap().at(i).unwrap(),
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            THE_TOLERANCE,
        );
        near(
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            0.0,
            THE_TOLERANCE,
        );
    }
    near(
        anOldSolver.minimum().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_NDim_ComparisonTest__IterationCount_BFGSVsPowell() {
    let mut aNewFunc: QuadraticFuncNew = QuadraticFuncNew::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 10.0;
    *aStart.change_at(1).unwrap() = 10.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 500;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;

    let aPowellResult: VectorResult = powell(&mut aNewFunc, &aStart, &aConfig);
    let aBFGSResult: VectorResult = bfgs(&mut aNewFunc, &aStart, &aConfig);

    assert!(aPowellResult.is_done());
    assert!(aBFGSResult.is_done());

    near(
        aPowellResult.value.unwrap(),
        aBFGSResult.value.unwrap(),
        THE_TOLERANCE,
    );

    assert!(aBFGSResult.iterations < aPowellResult.iterations);
}
