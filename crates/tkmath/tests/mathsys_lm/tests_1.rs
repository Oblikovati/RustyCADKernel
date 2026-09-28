#[test]
fn MathSys_LM_Test__RosenbrockResidual() {
    let mut aFunc: RosenbrockResidual = RosenbrockResidual;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = -1.0;
    aStart[2] = 1.0;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.f_tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 200;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_LOOSE_TOL,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        THE_LOOSE_TOL,
    );
    assert!(aResult.value.unwrap() < 0.01);
}

#[test]
fn MathSys_LM_Test__PowellSingular() {
    let mut aFunc: PowellSingular = PowellSingular;

    let mut aStart: Vector = Vector::with_bounds(1, 4).unwrap();
    aStart[1] = 3.0;
    aStart[2] = -1.0;
    aStart[3] = 0.0;
    aStart[4] = 1.0;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.f_tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 500;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());

    for i in 0..aResult.solution.as_ref().unwrap().length() as usize {
        near(*aResult.solution.as_ref().unwrap().at(i).unwrap(), 0.0, 0.1);
    }
    assert!(aResult.value.unwrap() < 0.01);
}
