#[test]
fn MathOpt_ComparisonTest__BFGSFasterThanPowellOnQuadratic() {
    let mut aFunc: SimpleQuadratic = SimpleQuadratic::default();
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 10.0;
    *aStart.change_at(1).unwrap() = 10.0;

    let mut aConfig: Config = Config::default();
    aConfig.max_iterations = 500;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-10;

    let aPowellResult: VectorResult = powell(&mut aFunc, &aStart, &aConfig);
    let aBFGSResult: VectorResult = bfgs(&mut aFunc, &aStart, &aConfig);

    assert!(aPowellResult.is_done());
    assert!(aBFGSResult.is_done());

    near(
        aPowellResult.value.unwrap(),
        aBFGSResult.value.unwrap(),
        1.0e-4,
    );

    assert!(aBFGSResult.iterations < aPowellResult.iterations);
}
