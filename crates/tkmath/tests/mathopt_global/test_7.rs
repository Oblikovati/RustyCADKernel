#[test]
fn MathOpt_GlobOptTest__DifferentialEvolutionRejectsInvalidCandidatesWithoutPoisoningBest() {
    let mut aFunc: PartiallyDefinedFunc = PartiallyDefinedFunc::default();
    let mut aLower: Vector = Vector::filled(0, 1 - 1, -1.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();
    let mut aConfig: GlobalConfig = GlobalConfig::new(GlobalStrategy::DifferentialEvolution, 100);
    aConfig.population = 20;
    aConfig.polish_budget_per_dim = 0;

    let aResult: VectorResult = differential_evolution(&mut aFunc, &aLower, &anUpper, &aConfig);

    assert!(aFunc.NbRejectedCalls > 0);
    assert_ne!(aResult.status, Status::CallbackError);
    assert_ne!(aResult.status, Status::NumericalError);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
}
