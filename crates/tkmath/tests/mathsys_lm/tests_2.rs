#[test]
fn MathSys_LM_Test__OverdeterminedSystem() {
    let mut aFunc: OverdeterminedSystem = OverdeterminedSystem;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 100;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
}

#[test]
fn MathSys_LM_Test__ExponentialFit() {
    let mut aFunc: ExponentialFit = ExponentialFit;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.5;
    aStart[2] = 0.5;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-8;
    aConfig.solver.max_iterations = 100;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());

    near(*aResult.solution.as_ref().unwrap().at(0).unwrap(), 1.0, 0.1);
    near(*aResult.solution.as_ref().unwrap().at(1).unwrap(), 1.0, 0.1);
}
