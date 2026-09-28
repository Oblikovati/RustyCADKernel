#[test]
fn MathOpt_PSOTest__MinIterationsUsesCompletedIterationCount() {
    let mut aFunc: SphereFunc = SphereFunc;
    let mut aLower: Vector = Vector::filled(0, 1 - 1, -1.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();
    let mut aSeedPos: Vector = Vector::filled(0, 1 - 1, 0.0).unwrap();
    let mut aSeeds: Vec<PSOSeedParticle> = vec![PSOSeedParticle::new(&aSeedPos)];

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 1;
    aConfig.solver.max_iterations = 10;
    aConfig.min_iterations = 3;
    aConfig.no_improve_iters = 1;
    aConfig.init_mode = PSOInitMode::SeededOnly;
    aConfig.velocity_clamp = 0.0;
    aConfig.phi_personal = 0.0;
    aConfig.phi_global = 0.0;
    aConfig.polish_budget_per_dim = 0;

    let aResult: VectorResult =
        pso_seeded(&mut aFunc, &aLower, &anUpper, &aConfig, Some(&aSeeds), None);

    assert_eq!(aResult.status, Status::Ok);
    assert_eq!(aResult.iterations, 4);
}
