#[test]
fn MathSys_LM_Test__LambdaUpdateFactors() {
    let mut aFunc: RosenbrockResidual = RosenbrockResidual;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = -1.0;
    aStart[2] = 1.0;

    let aFactors: [[f64; 2]; 4] = [[2.0, 0.5], [5.0, 0.2], [10.0, 0.1], [20.0, 0.05]];

    for aFactor in aFactors {
        let mut aConfig: LMConfig = LMConfig::default();
        aConfig.solver.tolerance = 1.0e-10;
        aConfig.solver.f_tolerance = 1.0e-10;
        aConfig.solver.max_iterations = 300;
        aConfig.lambda_increase = aFactor[0];
        aConfig.lambda_decrease = aFactor[1];

        let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

        assert!(aResult.is_done());
        assert!(aResult.value.unwrap() < 0.1);
    }
}

#[test]
fn MathSys_LM_Test__CompareWithOldAPI_CircleHyperbola() {
    let mut anOldFunc: CircleHyperbolaOld = CircleHyperbolaOld;
    let mut aNewFunc: CircleHyperbola = CircleHyperbola;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 1.5;
    aStart[2] = 0.5;

    let mut aTol: Vector = Vector::filled(1, 2, 1.0e-10).unwrap();

    let mut anOldSolver: FunctionSetRoot = FunctionSetRoot::new(&anOldFunc, &aTol, 100).unwrap();
    anOldSolver.perform(&mut anOldFunc, &aStart, false).unwrap();

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.f_tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 100;
    let mut aNewResult: SystemResult = levenberg_marquardt(&mut aNewFunc, &aStart, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    let anOldSol = anOldSolver.root().unwrap();
    near(
        anOldSol[1] * anOldSol[1] + anOldSol[2] * anOldSol[2],
        4.0,
        THE_LOOSE_TOL,
    );
    near(anOldSol[1] * anOldSol[2], 1.0, THE_LOOSE_TOL);

    let aNewX: f64 = *aNewResult.solution.as_ref().unwrap().at(0).unwrap();
    let aNewY: f64 = *aNewResult.solution.as_ref().unwrap().at(1).unwrap();
    near(aNewX * aNewX + aNewY * aNewY, 4.0, THE_TOLERANCE);
    near(aNewX * aNewY, 1.0, THE_TOLERANCE);
}
