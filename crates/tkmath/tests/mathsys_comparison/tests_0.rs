#[test]
fn MathSys_ComparisonTest__Newton_CircleLineSystem() {
    let mut aFunc: SimpleCircleLineSystem = SimpleCircleLineSystem;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.5;
    aStart[2] = 0.5;

    let mut aTolX: Vector = Vector::filled(1, 2, 1.0e-10).unwrap();
    let aTolF: f64 = 1.0e-10;

    let mut aNewResult: SystemResult = newton(&mut aFunc, &aStart, &aTolX, aTolF, 100);
    assert!(aNewResult.is_done());

    let mut aOldSolver: NewtonFunctionSetRoot =
        NewtonFunctionSetRoot::new(&aFunc, &aTolX, aTolF, 100).unwrap();
    aOldSolver.perform(&mut aFunc, &aStart).unwrap();
    assert!(aOldSolver.is_done());

    let aOldRoot = aOldSolver.root().unwrap();

    let aExpected: f64 = (0.5_f64).sqrt();

    near(
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        aExpected,
        THE_TOLERANCE,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        aExpected,
        THE_TOLERANCE,
    );
    near(aOldRoot[1], aExpected, THE_TOLERANCE);
    near(aOldRoot[2], aExpected, THE_TOLERANCE);

    near(
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        *aOldRoot.at(0).unwrap(),
        THE_TOLERANCE,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        *aOldRoot.at(1).unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathSys_ComparisonTest__Newton_CircleLineFromUnequalStart() {
    let mut aFunc: SimpleCircleLineSystem = SimpleCircleLineSystem;
    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.8;
    *aStart.change_at(1).unwrap() = 0.6;
    let mut aResult: SystemResult = newton_uniform(&mut aFunc, &aStart, 1.0e-13, 1.0e-13, 100);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.707_106_781_186_547_5,
        1.0e-12,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.707_106_781_186_547_5,
        1.0e-12,
    );
}
