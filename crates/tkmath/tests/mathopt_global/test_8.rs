#[test]
fn MathOpt_GlobOptTest__DifferentialEvolutionBudgetExhaustionRetainsBest() {
    let mut aFunc: SphereFunc = SphereFunc;
    let mut aLower: Vector = Vector::filled(0, 1 - 1, -1.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();
    let mut aConfig: GlobalConfig = GlobalConfig::new(GlobalStrategy::DifferentialEvolution, 1);
    aConfig.population = 4;
    aConfig.solver.tolerance = 1.0e-30;
    aConfig.polish_budget_per_dim = 0;

    let aResult: VectorResult = differential_evolution(&mut aFunc, &aLower, &anUpper, &aConfig);

    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
}
