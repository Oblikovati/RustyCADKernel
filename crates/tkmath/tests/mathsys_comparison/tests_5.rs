#[test]
fn MathSys_ComparisonTest__NewtonBoundedResolvesFreeVariablesAtActiveBound() {
    let mut aFunc: BoundaryTangentSystem = BoundaryTangentSystem;
    let mut aStart: Vector = Vector::filled(0, 2 - 1, 0.0).unwrap();
    let mut aLower: Vector = Vector::with_size(2);
    let mut anUpper: Vector = Vector::filled(0, 2 - 1, 2.0).unwrap();
    let mut aTolX: Vector = Vector::filled(0, 2 - 1, 1.0e-10).unwrap();
    *aLower.change_at(0).unwrap() = 0.0;
    *aLower.change_at(1).unwrap() = -2.0;

    let mut aResult: SystemResult =
        newton_bounded(&mut aFunc, &aStart, &aLower, &anUpper, &aTolX, 1.0e-10, 100);
    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        1.0e-12,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        1.0e-10,
    );
}

#[test]
fn MathSys_ComparisonTest__Newton_DifferentStartingPoints() {
    let mut aFunc: SimpleCircleLineSystem = SimpleCircleLineSystem;

    let mut aTolX: Vector = Vector::filled(1, 2, 1.0e-10).unwrap();
    let aTolF: f64 = 1.0e-10;

    let aStartPoints: [[f64; 2]; 4] = [[0.1, 0.1], [0.9, 0.9], [0.3, 0.7], [1.0, 0.0]];

    for aStartPt in aStartPoints {
        let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
        aStart[1] = aStartPt[0];
        aStart[2] = aStartPt[1];

        let mut aNewResult: SystemResult = newton(&mut aFunc, &aStart, &aTolX, aTolF, 100);
        assert!(aNewResult.is_done());

        let mut aOldSolver: NewtonFunctionSetRoot =
            NewtonFunctionSetRoot::new(&aFunc, &aTolX, aTolF, 100).unwrap();
        aOldSolver.perform(&mut aFunc, &aStart).unwrap();
        assert!(aOldSolver.is_done());

        let aOldRoot = aOldSolver.root().unwrap();

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
}
