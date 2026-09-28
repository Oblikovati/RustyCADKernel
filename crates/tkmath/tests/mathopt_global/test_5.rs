#[test]
fn MathOpt_GlobOptTest__DifferentialEvolution_Rosenbrock() {
    let mut aFunc: RosenbrockFunc = RosenbrockFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: GlobalConfig = GlobalConfig::default();
    aConfig.solver.max_iterations = 200;
    aConfig.population = 50;
    aConfig.mutation_scale = 0.8;
    aConfig.crossover_prob = 0.9;

    let aResult: VectorResult = differential_evolution(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());
    near(*aResult.solution.as_ref().unwrap().at(0).unwrap(), 1.0, 0.1);
    near(*aResult.solution.as_ref().unwrap().at(1).unwrap(), 1.0, 0.1);
    assert!(aResult.value.unwrap() < 0.1);
}
