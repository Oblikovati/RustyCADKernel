#[test]
fn MathOpt_PSOTest__BoundaryMode_Reflect_NearBoundMinimum() {
    let mut aFunc: BoothFunc = BoothFunc;

    let mut aLower: Vector = Vector::with_size(2);
    *aLower.change_at(0).unwrap() = 0.0;
    *aLower.change_at(1).unwrap() = 2.0;

    let mut aUpper: Vector = Vector::with_size(2);
    *aUpper.change_at(0).unwrap() = 2.0;
    *aUpper.change_at(1).unwrap() = 4.0;

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 40;
    aConfig.solver.max_iterations = 200;
    aConfig.solver.tolerance = 1.0e-6;
    aConfig.boundary_mode = PSOBoundaryMode::Reflect;

    let mut aStats: PSOStats = PSOStats::default();
    let aResult: VectorResult = pso_seeded(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfig,
        None,
        Some(&mut aStats),
    );

    assert!(aResult.is_done());
    near(*aResult.solution.as_ref().unwrap().at(0).unwrap(), 1.0, 0.1);
    near(*aResult.solution.as_ref().unwrap().at(1).unwrap(), 3.0, 0.1);
    assert!(aResult.value.unwrap() < 0.1);

    assert!(aStats.boundary_corrections > 0);
}
