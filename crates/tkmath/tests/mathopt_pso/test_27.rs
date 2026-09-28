#[test]
fn MathOpt_PSOTest__BudgetExhaustionRetainsBest() {
    let mut aFunc: SphereFunc = SphereFunc;
    let mut aLower: Vector = Vector::filled(0, 1 - 1, -1.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();
    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 4;
    aConfig.solver.max_iterations = 1;
    aConfig.no_improve_iters = 10;
    aConfig.polish_budget_per_dim = 0;

    let aResult: VectorResult = pso(&mut aFunc, &aLower, &anUpper, &aConfig);

    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
}
