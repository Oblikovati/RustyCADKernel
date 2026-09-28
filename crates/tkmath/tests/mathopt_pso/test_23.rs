#[test]
fn MathOpt_PSOTest__InBoundsSeed_ReEvaluates_StaleValue() {
    let mut aFunc: SphereFunc = SphereFunc;
    let mut aLower: Vector = Vector::filled(0, 1 - 1, -2.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 2.0).unwrap();
    let mut aSeedPos: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();

    let mut aSeeds: Vec<PSOSeedParticle> = vec![PSOSeedParticle::with_value(&aSeedPos, -100.0)];

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 1;
    aConfig.solver.max_iterations = 1;
    aConfig.init_mode = PSOInitMode::SeededOnly;
    aConfig.velocity_clamp = 0.0;
    aConfig.phi_personal = 0.0;
    aConfig.phi_global = 0.0;
    aConfig.no_improve_iters = 10;
    aConfig.polish_budget_per_dim = 0;

    let mut aStats: PSOStats = PSOStats::default();
    let aResult: VectorResult = pso_seeded(
        &mut aFunc,
        &aLower,
        &anUpper,
        &aConfig,
        Some(&aSeeds),
        Some(&mut aStats),
    );

    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.value.is_some());
    assert_eq!(aResult.value.unwrap(), 1.0);
    assert_eq!(aStats.function_evals, 2);
}
