#[test]
fn MathOpt_PSOTest__TargetValue_EarlyStop() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfigFull: PSOConfig = PSOConfig::default();
    aConfigFull.particles = 40;
    aConfigFull.solver.max_iterations = 200;
    aConfigFull.solver.tolerance = 1.0e-10;

    let mut aStatsFull: PSOStats = PSOStats::default();
    let aResultFull: VectorResult = pso_seeded(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfigFull,
        None,
        Some(&mut aStatsFull),
    );
    assert!(aResultFull.is_done());

    let mut aConfigTarget: PSOConfig = aConfigFull;
    aConfigTarget.target_value = Some(0.1);

    let mut aStatsTarget: PSOStats = PSOStats::default();
    let aResultTarget: VectorResult = pso_seeded(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfigTarget,
        None,
        Some(&mut aStatsTarget),
    );
    assert!(aResultTarget.is_done());

    assert!(aStatsTarget.iterations <= aStatsFull.iterations);
    assert!(aResultTarget.value.unwrap() <= 0.1);
}
