#[test]
fn MathSys_LM_Test__LinearSystem2D() {
    let mut aFunc: LinearSystem2D = LinearSystem2D;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.f_tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 100;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        2.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathSys_LM_Test__CircleHyperbola() {
    let mut aFunc: CircleHyperbola = CircleHyperbola;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 1.5;
    aStart[2] = 0.5;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-10;
    aConfig.solver.f_tolerance = 1.0e-10;
    aConfig.solver.max_iterations = 100;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());

    let aX: f64 = *aResult.solution.as_ref().unwrap().at(0).unwrap();
    let aY: f64 = *aResult.solution.as_ref().unwrap().at(1).unwrap();

    near(aX * aX + aY * aY, 4.0, THE_TOLERANCE);
    near(aX * aY, 1.0, THE_TOLERANCE);
}
