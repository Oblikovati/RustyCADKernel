#[test]
fn MathSys_ComparisonTest__Newton_VerifyFunctionValues() {
    let mut aFunc: SimpleCircleLineSystem = SimpleCircleLineSystem;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.5;
    aStart[2] = 0.5;

    let mut aResult: SystemResult = newton_uniform(&mut aFunc, &aStart, 1.0e-10, 1.0e-10, 100);
    assert!(aResult.is_done());

    let mut aF: Vector = Vector::with_bounds(1, 2).unwrap();
    assert!(aFunc.value(aResult.solution.as_ref().unwrap(), &mut aF));

    near(aF[1], 0.0, 1.0e-8);
    near(aF[2], 0.0, 1.0e-8);
}

#[test]
fn MathSys_ComparisonTest__IterationCount_NewVsOld() {
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

    assert!(aNewResult.iterations > 0);
    assert!(aNewResult.iterations <= 20);

    let aOldIter: i32 = aOldSolver.iterations().unwrap();
    near(aNewResult.iterations as i32 as f64, aOldIter as f64, 5.0);
}
