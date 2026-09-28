#[test]
fn MathOpt_PSOTest__Restart_Ackley_RecoversAfterStagnation() {
    let mut aFunc: AckleyFunc = AckleyFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 30;
    aConfig.solver.max_iterations = 300;
    aConfig.solver.tolerance = 1.0e-8;
    aConfig.restart_fraction = 0.5;
    aConfig.no_improve_iters = 5;

    let mut aStats: PSOStats = PSOStats::default();
    let aResult: VectorResult = pso_seeded(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfig,
        None,
        Some(&mut aStats),
    );

    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.value.is_some());
    assert!(aResult.value.unwrap() < 1.0);

    assert!(aStats.function_evals > 0);
    assert!(aStats.restarts > 0);
    assert!(aStats.stagnation_events > 0);
}
