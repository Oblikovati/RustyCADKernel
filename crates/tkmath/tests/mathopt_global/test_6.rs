#[test]
fn MathOpt_GlobOptTest__DifferentialEvolution_Rastrigin() {
    let mut aFunc: RastriginFunc = RastriginFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.12).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.12).unwrap();

    let mut aConfig: GlobalConfig = GlobalConfig::default();
    aConfig.solver.max_iterations = 200;
    aConfig.population = 60;
    aConfig.mutation_scale = 0.5;
    aConfig.crossover_prob = 0.9;

    let aResult: VectorResult = differential_evolution(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());

    near(*aResult.solution.as_ref().unwrap().at(0).unwrap(), 0.0, 0.5);
    near(*aResult.solution.as_ref().unwrap().at(1).unwrap(), 0.0, 0.5);
    assert!(aResult.value.unwrap() < 1.0);
}
