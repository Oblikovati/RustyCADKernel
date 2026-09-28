#[test]
fn MathSys_LM_Test__MaxIterationsReached() {
    let mut aFunc: LinearSystem2D = LinearSystem2D;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-15;
    aConfig.solver.x_tolerance = 1.0e-15;
    aConfig.solver.f_tolerance = 1.0e-15;
    aConfig.solver.relative_tolerance = 0.0;
    aConfig.solver.max_iterations = 1;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

    assert_eq!(aResult.status, Status::MaxIterations);
    assert_eq!(aResult.iterations, 1);
    assert!(aResult.solution.is_some());
}

#[test]
fn MathSys_LM_Test__InvalidInputDimensions() {
    let mut aFunc: LinearSystem2D = LinearSystem2D;

    let mut aStart: Vector = Vector::with_bounds(1, 3).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;
    aStart[3] = 0.0;

    let mut aConfig: LMConfig = LMConfig::default();

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

    assert!(!(aResult.is_done()));
    assert_eq!(aResult.status, Status::InvalidInput);
}
