#[test]
fn MathOpt_PSOTest__Stats_Reproducibility_FixedSeed() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 30;
    aConfig.solver.max_iterations = 50;
    aConfig.solver.tolerance = 1.0e-6;

    let mut aStats1: PSOStats = PSOStats::default();
    let aResult1: VectorResult = pso_seeded(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfig,
        None,
        Some(&mut aStats1),
    );

    let mut aStats2: PSOStats = PSOStats::default();
    let aResult2: VectorResult = pso_seeded(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfig,
        None,
        Some(&mut aStats2),
    );

    assert!(aResult1.is_done());
    assert!(aResult2.is_done());

    assert_eq!(aStats1.function_evals, aStats2.function_evals);
    assert_eq!(aStats1.iterations, aStats2.iterations);
    assert_eq!(aStats1.boundary_corrections, aStats2.boundary_corrections);
    assert!(aStats1.initial_best.is_some());
    assert!(aStats2.initial_best.is_some());
    assert!(aStats1.final_best.is_some());
    assert!(aStats2.final_best.is_some());
    assert_eq!(aStats1.initial_best.unwrap(), aStats2.initial_best.unwrap());
    assert_eq!(aStats1.final_best.unwrap(), aStats2.final_best.unwrap());
    assert_eq!(aResult1.value.unwrap(), aResult2.value.unwrap());
}
