#[test]
fn MathOpt_GlobOptTest__DifferentialEvolution_Sphere5D() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 5 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 5 - 1, 5.0).unwrap();

    let mut aConfig: GlobalConfig = GlobalConfig::default();
    aConfig.solver.max_iterations = 300;
    aConfig.population = 80;
    aConfig.mutation_scale = 0.7;
    aConfig.crossover_prob = 0.9;

    let aResult: VectorResult = differential_evolution(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());
    for i in 1..=5 {
        near(
            *aResult
                .solution
                .as_ref()
                .unwrap()
                .at((i - 1) as usize)
                .unwrap(),
            0.0,
            0.1,
        );
    }
    assert!(aResult.value.unwrap() < 0.1);
}
