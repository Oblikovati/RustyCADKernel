#[test]
fn MathSys_ComparisonTest__Newton_RosenbrockSystem() {
    let mut aFunc: RosenbrockSystem = RosenbrockSystem;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;

    let mut aTolX: Vector = Vector::filled(1, 2, 1.0e-10).unwrap();
    let aTolF: f64 = 1.0e-10;

    let mut aNewResult: SystemResult = newton(&mut aFunc, &aStart, &aTolX, aTolF, 100);
    assert!(aNewResult.is_done());

    let mut aOldSolver: NewtonFunctionSetRoot =
        NewtonFunctionSetRoot::new(&aFunc, &aTolX, aTolF, 100).unwrap();
    aOldSolver.perform(&mut aFunc, &aStart).unwrap();
    assert!(aOldSolver.is_done());

    let aOldRoot = aOldSolver.root().unwrap();

    near(
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(aOldRoot[1], 1.0, THE_TOLERANCE);
    near(aOldRoot[2], 1.0, THE_TOLERANCE);

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
fn MathSys_ComparisonTest__Newton_Linear3System() {
    let mut aFunc: Linear3System = Linear3System;

    let mut aStart: Vector = Vector::with_bounds(1, 3).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;
    aStart[3] = 0.0;

    let mut aTolX: Vector = Vector::filled(1, 3, 1.0e-10).unwrap();
    let aTolF: f64 = 1.0e-10;

    let mut aNewResult: SystemResult = newton(&mut aFunc, &aStart, &aTolX, aTolF, 100);
    assert!(aNewResult.is_done());

    let mut aOldSolver: NewtonFunctionSetRoot =
        NewtonFunctionSetRoot::new(&aFunc, &aTolX, aTolF, 100).unwrap();
    aOldSolver.perform(&mut aFunc, &aStart).unwrap();
    assert!(aOldSolver.is_done());

    let aOldRoot = aOldSolver.root().unwrap();

    near(
        *aNewResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.2,
        THE_TOLERANCE,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.6,
        THE_TOLERANCE,
    );
    near(
        *aNewResult.solution.as_ref().unwrap().at(2).unwrap(),
        1.0,
        THE_TOLERANCE,
    );

    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            *aOldRoot.at(i).unwrap(),
            THE_TOLERANCE,
        );
    }
}
