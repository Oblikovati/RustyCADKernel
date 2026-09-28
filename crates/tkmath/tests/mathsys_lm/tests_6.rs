#[test]
fn MathSys_LM_Test__CompareWithOldAPI_Rosenbrock() {
    let mut anOldFunc: RosenbrockResidualOld = RosenbrockResidualOld;
    let mut aNewFunc: RosenbrockResidual = RosenbrockResidual;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = -1.0;
    aStart[2] = 1.0;

    let mut aTol: Vector = Vector::filled(1, 2, 1.0e-10).unwrap();

    let mut anOldSolver: FunctionSetRoot = FunctionSetRoot::new(&anOldFunc, &aTol, 200).unwrap();
    anOldSolver.perform(&mut anOldFunc, &aStart, false).unwrap();

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.f_tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 200;
    let mut aNewResult: SystemResult = levenberg_marquardt(&mut aNewFunc, &aStart, &aConfig);

    assert!(aNewResult.is_done());

    near(
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_LOOSE_TOL,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        THE_LOOSE_TOL,
    );
}

#[test]
fn MathSys_LM_Test__CompareWithOldAPI_Bounded() {
    let mut anOldFunc: CircleHyperbolaOld = CircleHyperbolaOld;
    let mut aNewFunc: CircleHyperbola = CircleHyperbola;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 1.0;
    aStart[2] = 1.0;

    let mut aLower: Vector = Vector::with_bounds(1, 2).unwrap();
    aLower[1] = 0.0;
    aLower[2] = 0.0;

    let mut aUpper: Vector = Vector::with_bounds(1, 2).unwrap();
    aUpper[1] = 5.0;
    aUpper[2] = 5.0;

    let mut aTol: Vector = Vector::filled(1, 2, 1.0e-10).unwrap();

    let mut anOldSolver: FunctionSetRoot = FunctionSetRoot::new(&anOldFunc, &aTol, 100).unwrap();
    anOldSolver
        .perform_bounded(&mut anOldFunc, &aStart, &aLower, &aUpper, false)
        .unwrap();

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.f_tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 100;
    let mut aNewResult: SystemResult =
        levenberg_marquardt_bounded(&mut aNewFunc, &aStart, &aLower, &aUpper, &aConfig);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    let anOldSol = anOldSolver.root().unwrap();
    assert!(anOldSol[1] >= 0.0);
    assert!(anOldSol[1] <= 5.0);
    assert!(anOldSol[2] >= 0.0);
    assert!(anOldSol[2] <= 5.0);

    let aNewX: f64 = *aNewResult.solution.as_ref().unwrap().at(0).unwrap();
    let aNewY: f64 = *aNewResult.solution.as_ref().unwrap().at(1).unwrap();
    assert!(aNewX >= 0.0);
    assert!(aNewX <= 5.0);
    assert!(aNewY >= 0.0);
    assert!(aNewY <= 5.0);
}
