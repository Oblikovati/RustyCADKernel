#[test]
fn MathOpt_PSOTest__AsymmetricBounds() {
    let mut aFunc: BoothFunc = BoothFunc;

    let mut aLower: Vector = Vector::with_size(2);
    *aLower.change_at(0).unwrap() = -5.0;
    *aLower.change_at(1).unwrap() = 0.0;

    let mut aUpper: Vector = Vector::with_size(2);
    *aUpper.change_at(0).unwrap() = 5.0;
    *aUpper.change_at(1).unwrap() = 10.0;

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 40;
    aConfig.solver.max_iterations = 100;
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
