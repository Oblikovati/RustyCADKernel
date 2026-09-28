#[test]
fn MathOpt_PSOTest__Sphere3D() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 3 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 3 - 1, 5.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 40;
    aConfig.solver.max_iterations = 100;
    aConfig.solver.tolerance = 1.0e-6;

    let aResult: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult.is_done());
    for i in 1..=3 {
        near(
            *aResult
                .solution
                .as_ref()
                .unwrap()
                .at((i - 1) as usize)
                .unwrap(),
            0.0,
            THE_TOLERANCE,
        );
    }
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}
