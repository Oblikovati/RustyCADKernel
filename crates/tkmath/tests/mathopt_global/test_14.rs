#[test]
fn MathOpt_GlobOptTest__GoldsteinPrice() {
    let mut aFunc: GoldsteinPriceFunc = GoldsteinPriceFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -2.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 2.0).unwrap();

    let mut aConfig: GlobalConfig = GlobalConfig::new(GlobalStrategy::DifferentialEvolution, 200);
    aConfig.population = 60;

    let aResult: VectorResult = global_minimum(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());

    near(*aResult.solution.as_ref().unwrap().at(0).unwrap(), 0.0, 0.1);
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        -1.0,
        0.1,
    );
    near(aResult.value.unwrap(), 3.0, 0.5);
}
