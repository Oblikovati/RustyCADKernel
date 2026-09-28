#[test]
fn MathOpt_PSOTest__Sphere5D() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 5 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 5 - 1, 5.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 60;
    aConfig.solver.max_iterations = 200;
    aConfig.solver.tolerance = 1.0e-6;

    let aResult: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);

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
