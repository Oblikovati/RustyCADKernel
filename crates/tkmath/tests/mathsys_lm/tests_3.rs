#[test]
fn MathSys_LM_Test__BoundedCircleHyperbola() {
    let mut aFunc: CircleHyperbola = CircleHyperbola;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 1.0;
    aStart[2] = 1.0;

    let mut aLower: Vector = Vector::with_bounds(1, 2).unwrap();
    aLower[1] = 0.0;
    aLower[2] = 0.0;

    let mut aUpper: Vector = Vector::with_bounds(1, 2).unwrap();
    aUpper[1] = 5.0;
    aUpper[2] = 5.0;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.f_tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 100;

    let mut aResult: SystemResult =
        levenberg_marquardt_bounded(&mut aFunc, &aStart, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());

    let aX: f64 = *aResult.solution.as_ref().unwrap().at(0).unwrap();
    let aY: f64 = *aResult.solution.as_ref().unwrap().at(1).unwrap();

    assert!(aX >= 0.0);
    assert!(aX <= 5.0);
    assert!(aY >= 0.0);
    assert!(aY <= 5.0);

    near(aX * aX + aY * aY, 4.0, THE_TOLERANCE);
    near(aX * aY, 1.0, THE_TOLERANCE);
}

#[test]
fn MathSys_LM_Test__BoundedRosenbrock() {
    let mut aFunc: RosenbrockResidual = RosenbrockResidual;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = -1.0;
    aStart[2] = 1.0;

    let mut aLower: Vector = Vector::with_bounds(1, 2).unwrap();
    aLower[1] = -5.0;
    aLower[2] = -5.0;

    let mut aUpper: Vector = Vector::with_bounds(1, 2).unwrap();
    aUpper[1] = 5.0;
    aUpper[2] = 5.0;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.f_tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 200;

    let mut aResult: SystemResult =
        levenberg_marquardt_bounded(&mut aFunc, &aStart, &aLower, &aUpper, &aConfig);

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
}
