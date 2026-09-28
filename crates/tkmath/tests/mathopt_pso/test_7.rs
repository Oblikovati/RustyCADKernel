#[test]
fn MathOpt_PSOTest__ParameterSensitivity_Omega() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let aOmegas: [f64; 3] = [0.4, 0.7, 0.9];

    for aOmega in aOmegas {
        let mut aConfig: PSOConfig = PSOConfig::default();
        aConfig.particles = 30;
        aConfig.solver.max_iterations = 100;
        aConfig.solver.tolerance = 1.0e-6;
        aConfig.omega = aOmega;

        let aResult: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);

        assert!(aResult.is_done());
        assert!(aResult.value.unwrap() < 0.01);
    }
}
