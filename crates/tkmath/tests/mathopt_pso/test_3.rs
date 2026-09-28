#[test]
fn MathOpt_PSOTest__Booth() {
    let mut aFunc: BoothFunc = BoothFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -10.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 10.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 60;
    aConfig.solver.max_iterations = 200;
    aConfig.solver.tolerance = 1.0e-6;

    let aResult: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_STOCHASTIC_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        3.0,
        THE_STOCHASTIC_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_STOCHASTIC_TOLERANCE);
}
