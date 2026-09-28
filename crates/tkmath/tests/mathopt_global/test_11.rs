#[test]
fn MathOpt_GlobOptTest__MultiStart_Booth() {
    let mut aFunc: BoothFunc = BoothFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -10.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 10.0).unwrap();

    let mut aConfig: GlobalConfig = GlobalConfig::default();
    aConfig.starts = 30;

    let aResult: VectorResult = multi_start(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());
    near(*aResult.solution.as_ref().unwrap().at(0).unwrap(), 1.0, 0.5);
    near(*aResult.solution.as_ref().unwrap().at(1).unwrap(), 3.0, 0.5);
    assert!(aResult.value.unwrap() < 1.0);
}
