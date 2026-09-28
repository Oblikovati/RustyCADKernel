#[test]
fn MathOpt_FRPRTest__CompareWithOldAPI_Rosenbrock() {
    let mut anOldFunc: RosenbrockFuncOld = RosenbrockFuncOld;
    let mut aNewFunc: RosenbrockFunc = RosenbrockFunc::default();

    let mut anOldStart: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldStart[1] = -1.0;
    anOldStart[2] = 1.0;

    let mut aNewStart: Vector = Vector::with_size(2);
    *aNewStart.change_at(0).unwrap() = -1.0;
    *aNewStart.change_at(1).unwrap() = 1.0;

    let mut anOldSolver = Frpr::with_options(&anOldFunc, 1.0e-6, 2000, 1e-12).unwrap();
    anOldSolver.perform(&mut anOldFunc, &anOldStart).unwrap();

    let mut aConfig: FrprConfig = FrprConfig::default();
    aConfig.base.max_iterations = 2000;
    aConfig.base.tolerance = 1.0e-6;
    let aNewResult: VectorResult = frpr(&mut aNewFunc, &aNewStart, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    near(
        *anOldSolver.location().unwrap().at(0).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.01,
    );
    near(
        *anOldSolver.location().unwrap().at(1).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.01,
    );
}

#[test]
fn MathOpt_FRPRTest__CompareWithBFGS() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 100;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;

    let mut aFRPRConfig: FrprConfig = FrprConfig::default();
    aFRPRConfig.base.max_iterations = 100;
    aFRPRConfig.base.tolerance = 1.0e-8;

    let aFRPRResult: VectorResult = frpr(&mut aFunc, &aStart, &aFRPRConfig);
    let aBFGSResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert!(aFRPRResult.is_done());
    assert!(aBFGSResult.is_done());

    near(
        aFRPRResult.value.unwrap(),
        aBFGSResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(
        *aFRPRResult.solution.as_ref().unwrap().at(0).unwrap(),
        *aBFGSResult.solution.as_ref().unwrap().at(0).unwrap(),
        THE_TOLERANCE,
    );
    near(
        *aFRPRResult.solution.as_ref().unwrap().at(1).unwrap(),
        *aBFGSResult.solution.as_ref().unwrap().at(1).unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathOpt_FRPRTest__FormulaComparison() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aConfigFR: FrprConfig = FrprConfig::default();
    aConfigFR.base.max_iterations = 100;
    aConfigFR.base.tolerance = 1.0e-8;
    aConfigFR.formula = ConjugateGradientFormula::FletcherReeves;

    let mut aConfigPR: FrprConfig = FrprConfig::default();
    aConfigPR.base.max_iterations = 100;
    aConfigPR.base.tolerance = 1.0e-8;
    aConfigPR.formula = ConjugateGradientFormula::PolakRibiere;

    let mut aConfigHS: FrprConfig = FrprConfig::default();
    aConfigHS.base.max_iterations = 100;
    aConfigHS.base.tolerance = 1.0e-8;
    aConfigHS.formula = ConjugateGradientFormula::HestenesStiefel;

    let aResultFR: VectorResult = frpr(&mut aFunc, &aStart, &aConfigFR);
    let aResultPR: VectorResult = frpr(&mut aFunc, &aStart, &aConfigPR);
    let aResultHS: VectorResult = frpr(&mut aFunc, &aStart, &aConfigHS);

    assert!(aResultFR.is_done());
    assert!(aResultPR.is_done());
    assert!(aResultHS.is_done());

    near(
        aResultFR.value.unwrap(),
        aResultPR.value.unwrap(),
        THE_TOLERANCE,
    );
    near(
        aResultFR.value.unwrap(),
        aResultHS.value.unwrap(),
        THE_TOLERANCE,
    );
    near(
        *aResultFR.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aResultFR.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        THE_TOLERANCE,
    );
}
