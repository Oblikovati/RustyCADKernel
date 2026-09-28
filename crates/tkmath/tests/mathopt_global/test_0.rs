#[test]
fn MathOpt_GlobOptTest__GlobalMinimum_PSO_Sphere() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: GlobalConfig = GlobalConfig::new(GlobalStrategy::PSO, 100);
    aConfig.population = 40;

    let aResult: VectorResult = global_minimum(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        THE_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}
