#[test]
fn MathOpt_PSOTest__ContiguousStorageBeyondLocalVectorCapacity() {
    const A_NB_DIMS: usize = 40;
    let mut aFunc: SphereFunc = SphereFunc;
    let mut aLower: Vector = Vector::filled(0, (A_NB_DIMS) as i32 - 1, -1.0).unwrap();
    let mut aUpper: Vector = Vector::filled(0, (A_NB_DIMS) as i32 - 1, 1.0).unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 8;
    aConfig.solver.max_iterations = 1;
    aConfig.polish_budget_per_dim = 0;

    let aResult: VectorResult = pso(&mut aFunc, &aLower, &aUpper, &aConfig);

    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.solution.is_some());
    assert!(aResult.value.is_some());
    assert_eq!(
        aResult.solution.as_ref().unwrap().length() as usize,
        A_NB_DIMS
    );
    for aDimIdx in 0..A_NB_DIMS {
        assert!(
            *aResult.solution.as_ref().unwrap().at(aDimIdx).unwrap()
                >= *aLower.at(aDimIdx).unwrap()
        );
        assert!(
            *aResult.solution.as_ref().unwrap().at(aDimIdx).unwrap()
                <= *aUpper.at(aDimIdx).unwrap()
        );
    }
}
