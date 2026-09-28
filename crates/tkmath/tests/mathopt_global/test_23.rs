#[test]
fn MathOpt_GlobOptTest__GlobalMinimum_NonPSOStrategies_UnchangedWithPSOOptions() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfigMS: GlobalConfig = GlobalConfig::new(GlobalStrategy::MultiStart, 100);
    aConfigMS.starts = 20;

    let mut aPSOConfig: PSOConfig = PSOConfig::default();
    aPSOConfig.particles = 100;

    let aResultMS: VectorResult = global_minimum_with_pso(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfigMS,
        Some(&aPSOConfig),
        None,
        None,
    );
    assert!(aResultMS.is_done());
    near(aResultMS.value.unwrap(), 0.0, THE_TOLERANCE);

    let mut aConfigDE: GlobalConfig = GlobalConfig::new(GlobalStrategy::DifferentialEvolution, 100);
    aConfigDE.population = 40;

    let aResultDE: VectorResult = global_minimum_with_pso(
        &mut aFunc,
        &aLower,
        &aUpper,
        &aConfigDE,
        Some(&aPSOConfig),
        None,
        None,
    );
    assert!(aResultDE.is_done());
    near(aResultDE.value.unwrap(), 0.0, THE_TOLERANCE);
}
