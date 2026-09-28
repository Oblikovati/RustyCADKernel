#[test]
fn MathOpt_GlobOptTest__GlobalMinimum_PSO_CustomPSOConfig() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aGlobConfig: GlobalConfig = GlobalConfig::new(GlobalStrategy::PSO, 200);
    aGlobConfig.population = 40;

    let mut aPSOConfig: PSOConfig = PSOConfig::default();
    aPSOConfig.particles = 60;
    aPSOConfig.solver.max_iterations = 150;
    aPSOConfig.solver.tolerance = 1.0e-6;

    aPSOConfig.inertia_schedule = PSOInertiaSchedule::LinearDecay;
    aPSOConfig.omega_min = 0.3;

    let mut aStats: PSOStats = PSOStats::default();
    let aResult: VectorResult = global_minimum_with_pso(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aGlobConfig,
        Some(&aPSOConfig),
        None,
        Some(&mut aStats),
    );

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        THE_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
    assert!(aStats.function_evals > 0);
    assert!(aStats.iterations > 0);
}
