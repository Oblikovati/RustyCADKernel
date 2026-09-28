#[test]
fn MathOpt_PSOTest__SeededInit_Sphere2D_ImprovesEvalCount() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 40;
    aConfig.solver.max_iterations = 200;
    aConfig.solver.tolerance = 1.0e-6;

    let mut aStatsUnseeded: PSOStats = PSOStats::default();
    let aResultUnseeded: VectorResult = pso_seeded(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfig,
        None,
        Some(&mut aStatsUnseeded),
    );
    assert!(aResultUnseeded.is_done());

    let mut aConfigSeeded: PSOConfig = aConfig;
    aConfigSeeded.init_mode = PSOInitMode::SeededPlusRandom;

    let mut aSeedPos: Vector = Vector::filled(0, 2 - 1, 0.0).unwrap();
    *aSeedPos.change_at(0).unwrap() = 0.1;
    *aSeedPos.change_at(1).unwrap() = -0.1;
    let mut aSeeds: Vec<PSOSeedParticle> = vec![PSOSeedParticle::new(&aSeedPos)];

    let mut aStatsSeeded: PSOStats = PSOStats::default();
    let aResultSeeded: VectorResult = pso_seeded(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfigSeeded,
        Some(&aSeeds),
        Some(&mut aStatsSeeded),
    );
    assert!(aResultSeeded.is_done());

    assert!(aResultSeeded.value.unwrap() <= aResultUnseeded.value.unwrap() + 0.01);

    assert!(aStatsSeeded.function_evals > 0);
    assert!(aStatsSeeded.iterations > 0);
}
