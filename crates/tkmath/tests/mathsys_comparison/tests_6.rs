#[test]
fn MathSys_ComparisonTest__Newton_RectangularAndRankDeficientSystems() {
    let mut aStart: Vector = Vector::filled(1, 2, 0.0).unwrap();
    let mut anOverdetermined: RectangularSystem = RectangularSystem {
        myNbVariables: 2,
        myNbEquations: 3,
    };
    let mut anOverResult: SystemResult =
        newton_uniform(&mut anOverdetermined, &aStart, 1.0e-12, 1.0e-12, 100);
    assert!(anOverResult.is_done());
    assert_eq!(anOverResult.rank, 2);
    near(
        *anOverResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-10,
    );
    near(
        *anOverResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        1.0e-10,
    );

    let mut anUnderdetermined: RectangularSystem = RectangularSystem {
        myNbVariables: 2,
        myNbEquations: 1,
    };
    let mut anUnderResult: SystemResult =
        newton_uniform(&mut anUnderdetermined, &aStart, 1.0e-12, 1.0e-12, 100);
    assert!(anUnderResult.is_done());
    assert_eq!(anUnderResult.rank, 1);
    near(
        *anUnderResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.5,
        1.0e-10,
    );
    near(
        *anUnderResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.5,
        1.0e-10,
    );

    let mut aSingular: SingularSystem = SingularSystem;
    let mut aSingularResult: SystemResult =
        newton_uniform(&mut aSingular, &aStart, 1.0e-12, 1.0e-12, 100);
    assert!(aSingularResult.is_done());
    assert_eq!(aSingularResult.rank, 1);
    assert!(aSingularResult.is_root);
}

#[test]
fn MathSys_ComparisonTest__Newton_ArbitraryBoundsIndicesAndBoundStagnation() {
    let mut aSystem: RectangularSystem = RectangularSystem {
        myNbVariables: 2,
        myNbEquations: 3,
    };
    let mut aStart: Vector = Vector::filled(5, 6, 0.0).unwrap();
    let mut aTolerance: Vector = Vector::filled(5, 6, 1.0e-12).unwrap();
    let mut aLower: Vector = Vector::filled(-2, -1, -10.0).unwrap();
    let mut anUpper: Vector = Vector::filled(9, 10, 10.0).unwrap();
    let mut aResult: SystemResult = newton_bounded(
        &mut aSystem,
        &aStart,
        &aLower,
        &anUpper,
        &aTolerance,
        1.0e-12,
        100,
    );
    assert!(aResult.is_done());
    assert_eq!(aResult.jacobian.as_ref().unwrap().lower_col(), 0);
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        1.0e-10,
    );

    anUpper[9] = 0.5;
    let mut aBlocked: SystemResult = newton_bounded(
        &mut aSystem,
        &aStart,
        &aLower,
        &anUpper,
        &aTolerance,
        1.0e-12,
        20,
    );
    assert!(!(aBlocked.is_done()));
    assert!(!(aBlocked.is_root));
    assert!(aBlocked.residual_norm > 0.1);
}
