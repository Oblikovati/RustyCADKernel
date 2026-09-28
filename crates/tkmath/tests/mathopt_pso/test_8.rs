#[test]
fn MathOpt_PSOTest__ParameterSensitivity_SwarmSize() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let aSizes: [usize; 4] = [10, 30, 50, 100];

    for aSize in aSizes {
        let mut aConfig: PSOConfig = PSOConfig::default();
        aConfig.particles = aSize;
        aConfig.solver.max_iterations = 100;
        aConfig.solver.tolerance = 1.0e-6;

        let aResult: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);

        assert!(aResult.is_done());
        assert!(aResult.value.unwrap() < 0.01);
    }
}
