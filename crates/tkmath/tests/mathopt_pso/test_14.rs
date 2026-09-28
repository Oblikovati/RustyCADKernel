#[test]
fn MathOpt_PSOTest__SeededInit_Rastrigin2D_StableQuality() {
    let mut aFunc: RastriginFunc = RastriginFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.12).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.12).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 60;
    aConfig.solver.max_iterations = 200;
    aConfig.solver.tolerance = 1.0e-6;
    aConfig.init_mode = PSOInitMode::SeededPlusRandom;

    let mut aSeedPos: Vector = Vector::filled(0, 2 - 1, 0.0).unwrap();
    let mut aSeeds: Vec<PSOSeedParticle> = vec![PSOSeedParticle::with_value(&aSeedPos, 0.0)];

    let aResult: VectorResult =
        pso_seeded(&mut aFunc, &aLower, &aUpper, &aConfig, Some(&aSeeds), None);

    assert!(aResult.is_done());
    near(*aResult.solution.as_ref().unwrap().at(0).unwrap(), 0.0, 0.5);
    near(*aResult.solution.as_ref().unwrap().at(1).unwrap(), 0.0, 0.5);
    assert!(aResult.value.unwrap() < 1.0);
}
