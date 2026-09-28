#[test]
fn MathOpt_GlobOptTest__PSOHybridKeepsBetterPSOCandidateAfterClamping() {
    let mut aFunc: OutsideMinimumFunc = OutsideMinimumFunc::default();
    let mut aLower: Vector = Vector::filled(0, 1 - 1, 0.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();

    let mut aSeedPosition: Vector = Vector::filled(0, 1 - 1, 0.25).unwrap();
    let mut aSeeds: Vec<PSOSeedParticle> = vec![PSOSeedParticle::with_value(&aSeedPosition, 0.0)];
    let mut aPSOConfig: PSOConfig = PSOConfig::default();
    aPSOConfig.particles = 1;
    aPSOConfig.solver.max_iterations = 1;
    aPSOConfig.init_mode = PSOInitMode::SeededOnly;
    aPSOConfig.velocity_clamp = 0.0;
    aPSOConfig.phi_personal = 0.0;
    aPSOConfig.phi_global = 0.0;
    aPSOConfig.polish_budget_per_dim = 0;

    let mut aConfig: GlobalConfig = GlobalConfig::new(GlobalStrategy::PSOHybrid, 2);
    let aResult: VectorResult = global_minimum_with_pso(
        &mut aFunc,
        &aLower,
        &anUpper,
        &aConfig,
        Some(&aPSOConfig),
        Some(&aSeeds),
        None,
    );

    assert!(aResult.is_done());
    assert_eq!(*aResult.solution.as_ref().unwrap().at(0).unwrap(), 0.25);
    assert_eq!(aResult.value.unwrap(), 0.0);
    assert_eq!(aFunc.NbOutsideCalls, 0);
}
