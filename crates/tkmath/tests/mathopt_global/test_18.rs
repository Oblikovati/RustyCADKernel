#[test]
fn MathOpt_GlobOptTest__CompareWithOldAPI_Sphere() {
    let mut anOldFunc: SphereFuncOld = SphereFuncOld { myN: 2 };
    let mut aNewFunc: SphereFunc = SphereFunc;

    let mut anOldLower: Vector = Vector::filled(1, 2, -5.0).unwrap();
    let mut anOldUpper: Vector = Vector::filled(1, 2, 5.0).unwrap();

    let mut aNewLower: Vector = Vector::filled(0, 2 - 1, -5.0).unwrap();
    let mut aNewUpper: Vector = Vector::filled(0, 2 - 1, 5.0).unwrap();

    let mut anOldSolver = GlobalOptimum::with_options(
        GlobalObjective::Value(&mut anOldFunc),
        &anOldLower,
        &anOldUpper,
        0.01,
        1e-2,
        1e-7,
    )
    .unwrap();
    anOldSolver.perform(false).unwrap();
    assert!(anOldSolver.is_done());
    assert!(anOldSolver.number_of_extrema() > 0);
    let mut anOldSol: Vector = Vector::with_bounds(1, 2).unwrap();
    anOldSolver.points(1, &mut anOldSol).unwrap();

    let mut aConfig: GlobalConfig = GlobalConfig::new(GlobalStrategy::PSOHybrid, 100);
    aConfig.population = 40;

    let aNewResult: VectorResult = global_minimum(&mut aNewFunc, &aNewLower, &aNewUpper, &aConfig);

    assert!(aNewResult.is_done());

    assert!(anOldSolver.minimum() < 0.1);
    assert!(aNewResult.value.unwrap() < 0.1);
    near(anOldSolver.minimum(), aNewResult.value.unwrap(), 0.1);
    near(
        *anOldSol.at(0).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.1,
    );
    near(
        *anOldSol.at(1).unwrap(),
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.1,
    );
}
