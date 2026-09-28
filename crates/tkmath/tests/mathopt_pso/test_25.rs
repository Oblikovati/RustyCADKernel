#[test]
fn MathOpt_PSOTest__RejectedParticleDoesNotPoisonValidBest() {
    let mut aFunc: PartiallyDefinedFunc = PartiallyDefinedFunc::default();
    let mut aLower: Vector = Vector::filled(0, 1 - 1, -1.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();
    let mut aValidPos: Vector = Vector::filled(0, 1 - 1, 0.25).unwrap();
    let mut anInvalidPos: Vector = Vector::filled(0, 1 - 1, -0.25).unwrap();
    let mut aSeeds: Vec<PSOSeedParticle> = vec![
        PSOSeedParticle::new(&aValidPos),
        PSOSeedParticle::new(&anInvalidPos),
    ];

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 2;
    aConfig.solver.max_iterations = 3;
    aConfig.no_improve_iters = 1;
    aConfig.init_mode = PSOInitMode::SeededOnly;
    aConfig.velocity_clamp = 0.0;
    aConfig.phi_personal = 0.0;
    aConfig.phi_global = 0.0;
    aConfig.polish_budget_per_dim = 0;

    let aResult: VectorResult =
        pso_seeded(&mut aFunc, &aLower, &anUpper, &aConfig, Some(&aSeeds), None);

    assert!(aFunc.NbRejectedCalls > 0);
    assert!(aResult.is_done());
    assert_eq!(aResult.value.unwrap(), 0.0);
}
