#[test]
fn MathOpt_GlobOptTest__GlobalMinimum_PSOHybrid_WithSeeds() {
    let mut aFunc: BoothFunc = BoothFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -10.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 10.0).unwrap();

    let mut aGlobConfig: GlobalConfig = GlobalConfig::new(GlobalStrategy::PSOHybrid, 100);
    aGlobConfig.population = 40;

    let mut aPSOConfig: PSOConfig = PSOConfig::default();
    aPSOConfig.particles = 40;
    aPSOConfig.solver.max_iterations = 100;
    aPSOConfig.solver.tolerance = 1.0e-6;

    aPSOConfig.init_mode = PSOInitMode::SeededPlusRandom;

    let mut aSeedPos: Vector = Vector::with_size(2);
    *aSeedPos.change_at(0).unwrap() = 1.5;
    *aSeedPos.change_at(1).unwrap() = 2.5;
    let mut aSeeds: Vec<PSOSeedParticle> = vec![PSOSeedParticle::new(&aSeedPos)];
    let mut aStats: PSOStats = PSOStats::default();
    let aResult: VectorResult = global_minimum_with_pso(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aGlobConfig,
        Some(&aPSOConfig),
        Some(&aSeeds),
        Some(&mut aStats),
    );

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        3.0,
        THE_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
    assert!(aStats.function_evals > 0);
}
