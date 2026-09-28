#[test]
fn MathSys_LM_Test__NonFiniteResidualIsNumericalError() {
    let mut aFunc: NonFiniteResidual = NonFiniteResidual;
    let mut aStart: Vector = Vector::filled(1, 2, 0.0).unwrap();
    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &LMConfig::default());
    assert_eq!(aResult.status, Status::NumericalError);
    assert!(!(aResult.is_done()));
}

#[test]
fn MathSys_LM_Test__NonFiniteConfigIsInvalidInput() {
    let mut aFunc: LinearSystem2D = LinearSystem2D;
    let mut aStart: Vector = Vector::filled(1, 2, 0.0).unwrap();
    let mut aConfigs: [LMConfig; 11] = [LMConfig::default(); 11];
    let aNaN: f64 = f64::NAN;
    aConfigs[0].solver.tolerance = aNaN;
    aConfigs[1].solver.x_tolerance = aNaN;
    aConfigs[2].solver.f_tolerance = aNaN;
    aConfigs[3].solver.relative_tolerance = aNaN;
    aConfigs[4].solver.step_min = aNaN;
    aConfigs[5].lambda_init = aNaN;
    aConfigs[6].lambda_increase = aNaN;
    aConfigs[7].lambda_decrease = aNaN;
    aConfigs[8].lambda_max = aNaN;
    aConfigs[9].lambda_min = aNaN;
    aConfigs[10].rank_tolerance = aNaN;

    for aConfig in aConfigs {
        let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);
        assert_eq!(aResult.status, Status::InvalidInput);
    }
}
