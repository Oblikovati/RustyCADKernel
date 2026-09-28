#[test]
fn MathOpt_PSOTest__Reproducibility() {
    let mut aFunc: SphereFunc = SphereFunc;

    let mut aLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 30;
    aConfig.solver.max_iterations = 50;
    aConfig.solver.tolerance = 1.0e-6;

    let aResult1: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);
    let aResult2: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert!(aResult1.is_done());
    assert!(aResult2.is_done());

    assert_eq!(aResult1.value.unwrap(), aResult2.value.unwrap());
    assert_eq!(
        *aResult1.solution.as_ref().unwrap().at(0).unwrap(),
        *aResult2.solution.as_ref().unwrap().at(0).unwrap()
    );
    assert_eq!(
        *aResult1.solution.as_ref().unwrap().at(1).unwrap(),
        *aResult2.solution.as_ref().unwrap().at(1).unwrap()
    );
}
