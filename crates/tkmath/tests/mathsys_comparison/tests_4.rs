#[test]
fn MathSys_ComparisonTest__Newton_UniformTolerance() {
    let mut aFunc: SimpleCircleLineSystem = SimpleCircleLineSystem;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.5;
    aStart[2] = 0.5;

    let mut aResult: SystemResult = newton_uniform(&mut aFunc, &aStart, 1.0e-10, 1.0e-10, 100);
    assert!(aResult.is_done());

    let aExpected: f64 = (0.5_f64).sqrt();
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        aExpected,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        aExpected,
        THE_TOLERANCE,
    );
}

#[test]
fn MathSys_ComparisonTest__Newton_ComponentToleranceControlsConvergence() {
    let mut aFunc: DiagonalQuadraticSystem = DiagonalQuadraticSystem;
    let mut aStart: Vector = Vector::filled(0, 2 - 1, 0.0).unwrap();
    *aStart.change_at(0).unwrap() = 1.4;

    let mut aLooseTolX: Vector = Vector::filled(0, 2 - 1, 1.0e-6).unwrap();
    *aLooseTolX.change_at(0).unwrap() = 2.0e-2;
    let mut aLooseResult: SystemResult = newton(&mut aFunc, &aStart, &aLooseTolX, 1.0e-3, 10);
    assert!(aLooseResult.is_done());

    let mut aTightTolX: Vector = Vector::filled(0, 2 - 1, 1.0e-6).unwrap();
    *aTightTolX.change_at(0).unwrap() = 1.0e-5;
    let mut aTightResult: SystemResult = newton(&mut aFunc, &aStart, &aTightTolX, 1.0e-3, 10);
    assert!(aTightResult.is_done());
    assert!(aTightResult.iterations > aLooseResult.iterations);
    assert!(aTightResult.residual_norm < aLooseResult.residual_norm);

    let mut aRoot: Vector = Vector::filled(0, 2 - 1, 0.0).unwrap();
    *aRoot.change_at(0).unwrap() = (2.0_f64).sqrt();
    let mut anInitialRoot: SystemResult = newton(&mut aFunc, &aRoot, &aTightTolX, 1.0e-12, 10);
    assert!(anInitialRoot.is_done());
    assert_eq!(anInitialRoot.iterations, 0);
}
