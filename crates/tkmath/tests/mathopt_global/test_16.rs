#[test]
fn MathOpt_GlobOptTest__StrategyComparison_Rosenbrock() {
    let mut aFunc: RosenbrockFunc = RosenbrockFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfigPSO: GlobalConfig = GlobalConfig::new(GlobalStrategy::PSO, 200);
    aConfigPSO.population = 50;

    let aResultPSO: VectorResult = global_minimum(&mut aFunc, &aLower, &aUpper, &aConfigPSO);

    let mut aConfigDE: GlobalConfig = GlobalConfig::new(GlobalStrategy::DifferentialEvolution, 200);
    aConfigDE.population = 50;

    let aResultDE: VectorResult = global_minimum(&mut aFunc, &aLower, &aUpper, &aConfigDE);

    assert!(aResultPSO.is_done());
    assert!(aResultDE.is_done());

    assert!(aResultPSO.value.unwrap() < 1.0);
    assert!(aResultDE.value.unwrap() < 1.0);
}
