#[test]
fn MathSys_ComparisonTest__Newton_PolynomialSystem() {
    let mut aFunc: PolynomialSystem = PolynomialSystem;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.9;
    aStart[2] = 0.9;

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
fn MathSys_ComparisonTest__NewtonBounded_CircleLineSystem() {
    let mut aFunc: SimpleCircleLineSystem = SimpleCircleLineSystem;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.5;
    aStart[2] = 0.5;

    let mut aInfBound: Vector = Vector::with_bounds(1, 2).unwrap();
    aInfBound[1] = 0.0;
    aInfBound[2] = 0.0;

    let mut aSupBound: Vector = Vector::with_bounds(1, 2).unwrap();
    aSupBound[1] = 2.0;
    aSupBound[2] = 2.0;

    let mut aTolX: Vector = Vector::filled(1, 2, 1.0e-10).unwrap();
    let aTolF: f64 = 1.0e-10;

    let mut aNewResult: SystemResult = newton_bounded(
        &mut aFunc, &aStart, &aInfBound, &aSupBound, &aTolX, aTolF, 100,
    );
    assert!(aNewResult.is_done());

    let mut aOldSolver: NewtonFunctionSetRoot =
        NewtonFunctionSetRoot::new(&aFunc, &aTolX, aTolF, 100).unwrap();
    aOldSolver
        .perform_bounded(&mut aFunc, &aStart, &aInfBound, &aSupBound)
        .unwrap();
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
