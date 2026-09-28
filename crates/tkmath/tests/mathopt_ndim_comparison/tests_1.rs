#[test]
fn MathOpt_NDim_ComparisonTest__Powell_Sphere3D() {
    let mut anOldFunc: SphereFuncOld = SphereFuncOld { myN: 3 };
    let mut aNewFunc: SphereFuncNew = SphereFuncNew::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 3).unwrap();
    anOldStart[1] = 3.0;
    anOldStart[2] = -2.0;
    anOldStart[3] = 4.0;

    let mut aNewStart: Vector = Vector::with_size(3);
    *aNewStart.change_at(0).unwrap() = 3.0;
    *aNewStart.change_at(1).unwrap() = -2.0;
    *aNewStart.change_at(2).unwrap() = 4.0;

    let mut anOldSolver = Powell::with_options(&anOldFunc, 1.0e-8, 200, 1e-12).unwrap();
    let aDirections: Matrix = CreateIdentityMatrix(3);
    anOldSolver
        .perform(&mut anOldFunc, &anOldStart, &aDirections)
        .unwrap();

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 200;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;
    let aNewResult: VectorResult = powell(&mut aNewFunc, &aNewStart, &aConfig);

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
fn MathOpt_NDim_ComparisonTest__BFGS_Quadratic() {
    let mut anOldFunc: QuadraticFuncGradOld = QuadraticFuncGradOld::default();
    let mut aNewFunc: QuadraticFuncNew = QuadraticFuncNew::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldStart[1] = 5.0;
    anOldStart[2] = 7.0;

    let mut aNewStart: Vector = Vector::with_size(2);
    *aNewStart.change_at(0).unwrap() = 5.0;
    *aNewStart.change_at(1).unwrap() = 7.0;

    let mut anOldSolver = Bfgs::with_options(2, 1.0e-8, 100, 1e-12).unwrap();
    anOldSolver.perform(&mut anOldFunc, &anOldStart).unwrap();

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;
    let aNewResult: VectorResult = bfgs(&mut aNewFunc, &aNewStart, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        *anOldSolver.location().unwrap().at(0).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        THE_TOLERANCE,
    );
    near(
        *anOldSolver.location().unwrap().at(1).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        THE_TOLERANCE,
    );
    near(
        anOldSolver.minimum().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );

    near(
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_NDim_ComparisonTest__BFGS_Booth() {
    let mut anOldFunc: BoothFuncGradOld = BoothFuncGradOld::default();
    let mut aNewFunc: BoothFuncNew = BoothFuncNew::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldStart[1] = 0.0;
    anOldStart[2] = 0.0;

    let mut aNewStart: Vector = Vector::with_size(2);
    *aNewStart.change_at(0).unwrap() = 0.0;
    *aNewStart.change_at(1).unwrap() = 0.0;

    let mut anOldSolver = Bfgs::with_options(2, 1.0e-8, 100, 1e-12).unwrap();
    anOldSolver.perform(&mut anOldFunc, &anOldStart).unwrap();

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;
    let aNewResult: VectorResult = bfgs(&mut aNewFunc, &aNewStart, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        *anOldSolver.location().unwrap().at(0).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        THE_TOLERANCE,
    );
    near(
        *anOldSolver.location().unwrap().at(1).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        THE_TOLERANCE,
    );
    near(
        anOldSolver.minimum().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );

    near(
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        3.0,
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), 0.0, THE_TOLERANCE);
}
