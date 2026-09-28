#[test]
fn MathOpt_PSOTest__SeededOnly_NoSeeds_ReturnsInvalidInput() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 20;
    aConfig.solver.max_iterations = 50;
    aConfig.init_mode = PSOInitMode::SeededOnly;

    let aResult: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(!(aResult.is_done()));
    assert_eq!(aResult.status, Status::InvalidInput);
}
