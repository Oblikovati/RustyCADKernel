#[test]
fn MathOpt_PSOTest__ClampedSeed_ReEvaluates_StaleValue() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 10;
    aConfig.solver.max_iterations = 100;
    aConfig.solver.tolerance = 1.0e-8;
    aConfig.init_mode = PSOInitMode::SeededPlusRandom;

    let mut aSeedPos: Vector = Vector::with_size(2);
    *aSeedPos.change_at(0).unwrap() = 10.0;
    *aSeedPos.change_at(1).unwrap() = -10.0;

    let mut aSeeds: Vec<PSOSeedParticle> = Vec::new();
    let aSeed: PSOSeedParticle = PSOSeedParticle::with_value(&aSeedPos, 0.0);
    aSeeds.push(aSeed);

    let aResult: VectorResult =
        pso_seeded(&mut aFunc, &aLower, &aUpper, &aConfig, Some(&aSeeds), None);

    assert!(aResult.is_done());

    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        THE_STOCHASTIC_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        THE_STOCHASTIC_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_STOCHASTIC_TOLERANCE);
}
