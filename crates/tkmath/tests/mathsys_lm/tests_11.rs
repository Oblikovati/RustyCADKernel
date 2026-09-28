#[test]
fn MathSys_LM_Test__ReversedLambdaBoundsAreInvalidInput() {
    let mut aFunc: LinearSystem2D = LinearSystem2D;
    let mut aStart: Vector = Vector::filled(1, 2, 0.0).unwrap();
    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.lambda_min = 3.0;
    aConfig.lambda_init = 1.0;
    aConfig.lambda_max = 2.0;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);
    assert_eq!(aResult.status, Status::InvalidInput);
}

#[test]
fn MathSys_LM_Test__RelativeToleranceParticipatesInResidualConvergence() {
    let mut aFunc: LinearSystem2D = LinearSystem2D;
    let mut aStart: Vector = Vector::filled(1, 2, 0.0).unwrap();
    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.max_iterations = 1;
    aConfig.solver.f_tolerance = 1.0e-15;
    aConfig.solver.x_tolerance = 1.0e-15;
    aConfig.solver.relative_tolerance = 1.0e-2;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);
    assert!(aResult.is_done());
    assert!(aResult.is_root);
    assert_eq!(aResult.iterations, 1);
}
