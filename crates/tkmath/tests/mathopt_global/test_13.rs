#[test]
fn MathOpt_GlobOptTest__Himmelblau_FindsOneMinimum() {
    let mut aFunc: HimmelblauFunc = HimmelblauFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: GlobalConfig = GlobalConfig::new(GlobalStrategy::DifferentialEvolution, 200);
    aConfig.population = 50;

    let aResult: VectorResult = global_minimum(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());

    near(aResult.value.unwrap(), 0.0, THE_STOCHASTIC_TOLERANCE);
}
