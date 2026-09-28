#[test]
fn MathOpt_NDim_ComparisonTest__IterationCount_LBFGSVsBFGS() {
    let mut aNewFunc: SphereFuncNew = SphereFuncNew::default();

    let mut aStart: Vector = Vector::with_size(5);
    for i in 0..aStart.length() as usize {
        *aStart.change_at(i).unwrap() = (i + 1) as f64;
    }

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;

    let aBFGSResult: VectorResult = bfgs(&mut aNewFunc, &aStart, &aConfig);
    let aLBFGSResult: VectorResult = lbfgs(&mut aNewFunc, &aStart, 10, &aConfig);

    assert!(aBFGSResult.is_done());
    assert!(aLBFGSResult.is_done());

    near(
        aBFGSResult.value.unwrap(),
        aLBFGSResult.value.unwrap(),
        THE_TOLERANCE,
    );

    for i in 0..aBFGSResult.solution.as_ref().unwrap().length() as usize {
        near(
            *aBFGSResult.solution.as_ref().unwrap().at(i).unwrap(),
            0.0,
            THE_TOLERANCE,
        );
        near(
            *aLBFGSResult.solution.as_ref().unwrap().at(i).unwrap(),
            0.0,
            THE_TOLERANCE,
        );
    }
}

#[test]
fn MathOpt_NDim_ComparisonTest__Accuracy_BFGSVsOld() {
    let mut anOldFunc: QuadraticFuncGradOld = QuadraticFuncGradOld::default();
    let mut aNewFunc: QuadraticFuncNew = QuadraticFuncNew::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldStart[1] = 5.0;
    anOldStart[2] = 7.0;

    let mut aNewStart: Vector = Vector::with_size(2);
    *aNewStart.change_at(0).unwrap() = 5.0;
    *aNewStart.change_at(1).unwrap() = 7.0;

    let aTightTol: f64 = 1.0e-12;

    let mut anOldSolver = Bfgs::with_options(2, aTightTol, 200, 1e-12).unwrap();
    anOldSolver.perform(&mut anOldFunc, &anOldStart).unwrap();

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 200;
    aConfig.x_tolerance = aTightTol;
    aConfig.f_tolerance = 1.0e-14;
    let aNewResult: VectorResult = bfgs(&mut aNewFunc, &aNewStart, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(anOldSolver.location().unwrap()[1], 1.0, 1.0e-8);
    near(anOldSolver.location().unwrap()[2], 2.0, 1.0e-8);
    near(
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-8,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        1.0e-8,
    );
}

#[test]
fn MathOpt_NDim_ComparisonTest__DifferentStartingPoints() {
    let mut anOldFunc: QuadraticFuncGradOld = QuadraticFuncGradOld::default();
    let mut aNewFunc: QuadraticFuncNew = QuadraticFuncNew::default();

    let aStartPoints = [(0.0, 0.0), (10.0, 10.0), (-5.0, 5.0), (100.0, -50.0)];

    for aPoint in aStartPoints {
        let mut anOldStart: Vector = Vector::with_bounds(1, 2).unwrap();
        anOldStart[1] = aPoint.0;
        anOldStart[2] = aPoint.1;

        let mut aNewStart: Vector = Vector::with_size(2);
        *aNewStart.change_at(0).unwrap() = aPoint.0;
        *aNewStart.change_at(1).unwrap() = aPoint.1;

        let mut anOldSolver = Bfgs::with_options(2, 1.0e-8, 200, 1e-12).unwrap();
        anOldSolver.perform(&mut anOldFunc, &anOldStart).unwrap();

        let mut aConfig: Config = Config::default();
        aConfig.max_iterations = 200;
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
            *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
            1.0,
            THE_TOLERANCE,
        );
        near(
            *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
            2.0,
            THE_TOLERANCE,
        );
    }
}
