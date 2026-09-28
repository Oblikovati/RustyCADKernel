#[test]
fn MathOpt_PSOTest__CompareWithOldAPI_Sphere() {
    let mut anOldFunc: SphereFuncOld = SphereFuncOld { myN: 2 };
    let mut aNewFunc: SphereFunc = SphereFunc;

    let mut anOldLower: Vector = Vector::filled(1, 2, -5.0).unwrap();
    let mut anOldUpper: Vector = Vector::filled(1, 2, 5.0).unwrap();

    let mut aNewLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aNewUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut aStep: Vector = Vector::filled(1, 2, 0.5).unwrap();
    let mut anOldSolver =
        Pso::with_options(&mut anOldFunc, &anOldLower, &anOldUpper, &aStep, 30, 100).unwrap();
    let mut anOldValue: f64 = 0.0;
    let mut anOldOutPnt: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldSolver
        .perform(&aStep, &mut anOldValue, &mut anOldOutPnt)
        .unwrap();

    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 30;
    aConfig.solver.max_iterations = 100;
    aConfig.solver.tolerance = 1.0e-8;
    let aNewResult: VectorResult = pso(&mut aNewFunc, &aNewLower, &aNewUpper, &aConfig);

    assert!(aNewResult.is_done());

    assert!(anOldValue < 0.01);
    assert!(aNewResult.value.unwrap() < 0.01);
    near(anOldValue, aNewResult.value.unwrap(), 0.01);
    near(
        *anOldOutPnt.at(0).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.1,
    );
    near(
        *anOldOutPnt.at(1).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.1,
    );
}
