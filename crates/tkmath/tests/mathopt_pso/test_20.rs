#[test]
fn MathOpt_PSOTest__Polishing_Sphere2D_ReachesHighPrecision() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 30;
    aConfig.solver.max_iterations = 50;
    aConfig.solver.tolerance = 1.0e-10;

    let aResult: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());

    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        1.0e-8,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        1.0e-8,
    );
    near(aResult.value.unwrap(), 0.0, 1.0e-14);
}
