#[test]
fn MathLin_Crout_Test__SymmetricIndefiniteUsesLowerTriangle() {
    let mut aMatrix: Matrix = Matrix::filled(0, 2 - 1, 0, 2 - 1, 0.0).unwrap();
    *aMatrix.change_at(0_usize, 0_usize).unwrap() = 1.0;
    *aMatrix.change_at(1_usize, 0_usize).unwrap() = 2.0;
    *aMatrix.change_at(1_usize, 1_usize).unwrap() = 1.0;
    *aMatrix.change_at(0_usize, 1_usize).unwrap() = f64::NAN;

    let aResult = crout(&aMatrix, 1e-15);
    assert!(aResult.is_done());
    assert!(aResult.inverse.is_some());
    near(
        *aResult
            .inverse
            .as_ref()
            .unwrap()
            .at(0_usize, 0_usize)
            .unwrap(),
        -1.0 / 3.0,
        THE_TOLERANCE,
    );
    near(
        *aResult
            .inverse
            .as_ref()
            .unwrap()
            .at(0_usize, 1_usize)
            .unwrap(),
        2.0 / 3.0,
        THE_TOLERANCE,
    );
    near(
        *aResult
            .inverse
            .as_ref()
            .unwrap()
            .at(1_usize, 0_usize)
            .unwrap(),
        2.0 / 3.0,
        THE_TOLERANCE,
    );
    near(
        *aResult
            .inverse
            .as_ref()
            .unwrap()
            .at(1_usize, 1_usize)
            .unwrap(),
        -1.0 / 3.0,
        THE_TOLERANCE,
    );
}

#[test]
fn MathLin_Crout_Test__ScalingAndDeterminantOverflowDoNotInvalidateResults() {
    let mut aScaled: Matrix = Matrix::filled(0, 2 - 1, 0, 2 - 1, 0.0).unwrap();
    *aScaled.change_at(0_usize, 0_usize).unwrap() = 1.0e20;
    *aScaled.change_at(1_usize, 1_usize).unwrap() = 1.0;
    let mut aB: Vector = Vector::with_size(2usize);
    *aB.change_at(0_usize).unwrap() = 1.0e20;
    *aB.change_at(1_usize).unwrap() = 1.0;
    let aScaledSolve = solve_crout(&aScaled, &aB, 1e-15);
    assert!(aScaledSolve.is_done());
    assert_eq!(
        *aScaledSolve.solution.as_ref().unwrap().at(0_usize).unwrap(),
        1.0
    );
    assert_eq!(
        *aScaledSolve.solution.as_ref().unwrap().at(1_usize).unwrap(),
        1.0
    );

    let mut anOverflow: Matrix = Matrix::filled(0, 2 - 1, 0, 2 - 1, 0.0).unwrap();
    *anOverflow.change_at(0_usize, 0_usize).unwrap() = 1.0e200;
    *anOverflow.change_at(1_usize, 1_usize).unwrap() = 1.0e200;
    let anInverse = invert_crout(&anOverflow, 1e-15);
    assert!(anInverse.is_done());
    assert!(anInverse.inverse.is_some());
    assert_eq!(
        *anInverse
            .inverse
            .as_ref()
            .unwrap()
            .at(0_usize, 0_usize)
            .unwrap(),
        1.0e-200
    );
    assert_eq!(
        *anInverse
            .inverse
            .as_ref()
            .unwrap()
            .at(1_usize, 1_usize)
            .unwrap(),
        1.0e-200
    );
    assert!(!(anInverse.determinant.is_some()));
}
