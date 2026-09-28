#[test]
fn MathSys_LM_Test__BoundedActiveConstraint() {
    let mut aFunc: LinearSystem2D = LinearSystem2D;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 1.0;
    aStart[2] = 1.0;

    let mut aLower: Vector = Vector::with_bounds(1, 2).unwrap();
    aLower[1] = -10.0;
    aLower[2] = -10.0;

    let mut aUpper: Vector = Vector::with_bounds(1, 2).unwrap();
    aUpper[1] = 1.5;
    aUpper[2] = 10.0;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 100;

    let mut aResult: SystemResult =
        levenberg_marquardt_bounded(&mut aFunc, &aStart, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    assert!(aResult.is_stationary);
    assert!(!(aResult.is_root));
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.5,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    assert!(aResult.gradient.is_some());
    assert!(*aResult.gradient.as_ref().unwrap().at(0).unwrap() < -0.9);
    near(
        *aResult.gradient.as_ref().unwrap().at(1).unwrap(),
        0.0,
        THE_TOLERANCE,
    );
}

#[test]
fn MathSys_LM_Test__DampingParameters() {
    let mut aFunc: CircleHyperbola = CircleHyperbola;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 1.5;
    aStart[2] = 0.5;

    let aLambdas: [f64; 4] = [1.0e-6, 1.0e-3, 1.0, 100.0];

    for aLambda in aLambdas {
        let mut aConfig: LMConfig = LMConfig::default();
        aConfig.solver.tolerance = 1.0e-10;
        aConfig.solver.f_tolerance = 1.0e-10;
        aConfig.solver.max_iterations = 200;
        aConfig.lambda_init = aLambda;

        let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

        assert!(aResult.is_done());

        let aX: f64 = *aResult.solution.as_ref().unwrap().at(0).unwrap();
        let aY: f64 = *aResult.solution.as_ref().unwrap().at(1).unwrap();

        near(aX * aX + aY * aY, 4.0, THE_TOLERANCE);
        near(aX * aY, 1.0, THE_TOLERANCE);
    }
}
