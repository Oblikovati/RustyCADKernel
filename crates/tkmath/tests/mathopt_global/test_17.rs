#[test]
fn MathOpt_GlobOptTest__Reproducibility_DE() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: GlobalConfig = GlobalConfig::default();
    aConfig.solver.max_iterations = 100;
    aConfig.population = 40;

    let aResult1: VectorResult = differential_evolution(&mut aFunc, &aLower, &aUpper, &aConfig);
    let aResult2: VectorResult = differential_evolution(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult1.is_done());
    assert!(aResult2.is_done());

    assert_eq!(aResult1.value.unwrap(), aResult2.value.unwrap());
}
