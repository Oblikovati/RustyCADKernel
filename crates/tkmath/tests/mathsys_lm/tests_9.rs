#[test]
fn MathSys_LM_Test__BoundedInvalidDimensions() {
    let mut aFunc: LinearSystem2D = LinearSystem2D;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;

    let mut aLower: Vector = Vector::with_bounds(1, 3).unwrap();
    aLower[1] = -10.0;
    aLower[2] = -10.0;
    aLower[3] = -10.0;

    let mut aUpper: Vector = Vector::with_bounds(1, 2).unwrap();
    aUpper[1] = 10.0;
    aUpper[2] = 10.0;

    let mut aConfig: LMConfig = LMConfig::default();

    let mut aResult: SystemResult =
        levenberg_marquardt_bounded(&mut aFunc, &aStart, &aLower, &aUpper, &aConfig);

    assert!(!(aResult.is_done()));
    assert_eq!(aResult.status, Status::InvalidInput);
}

#[test]
fn MathSys_LM_Test__StationaryLargeResidualIsNotReportedAsRoot() {
    let mut aFunc: StationaryNonRoot = StationaryNonRoot;
    let mut aStart: Vector = Vector::filled(4, 5, 0.0).unwrap();
    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &LMConfig::default());
    assert!(aResult.is_done());
    assert!(aResult.is_stationary);
    assert!(!(aResult.is_root));
    assert_eq!(aResult.rank, 0);
    assert!(aResult.residual_norm > 100.0);
    assert!(aResult.jacobian.is_some());
    assert_eq!(aResult.jacobian.as_ref().unwrap().lower_col(), 0);
}
