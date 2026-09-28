#[test]
fn MathLin_Gauss_Test__DeterminantOverflowDoesNotInvalidateSolve() {
    let mut aA: Matrix = Matrix::filled(0, 2 - 1, 0, 2 - 1, 0.0).unwrap();
    *aA.change_at(0_usize, 0_usize).unwrap() = 1.0e200;
    *aA.change_at(1_usize, 1_usize).unwrap() = 1.0e200;

    let aLU = lu(&aA, 1e-15);
    assert!(aLU.is_done());
    assert!(!(aLU.determinant.is_some()));

    let mut aB: Vector = Vector::with_size(2usize);
    *aB.change_at(0_usize).unwrap() = 1.0e200;
    *aB.change_at(1_usize).unwrap() = 1.0e200;
    let aSolve = solve(&aA, &aB, 1e-15);
    assert!(aSolve.is_done());
    assert!(aSolve.solution.is_some());
    assert_eq!(*aSolve.solution.as_ref().unwrap().at(0_usize).unwrap(), 1.0);
    assert_eq!(*aSolve.solution.as_ref().unwrap().at(1_usize).unwrap(), 1.0);
    assert!(!(aSolve.determinant.is_some()));

    let aDeterminant = determinant(&aA, 1e-15);
    assert_eq!(aDeterminant.status, Status::NumericalError);
    assert!(!(aDeterminant.determinant.is_some()));
}

#[test]
fn MathLin_Gauss_Test__IndependentlyScaledRowsRemainSolvable() {
    let mut aA: Matrix = Matrix::filled(0, 2 - 1, 0, 2 - 1, 0.0).unwrap();
    *aA.change_at(0_usize, 0_usize).unwrap() = 1.0e20;
    *aA.change_at(1_usize, 1_usize).unwrap() = 1.0;
    let mut aB: Vector = Vector::with_size(2usize);
    *aB.change_at(0_usize).unwrap() = 1.0e20;
    *aB.change_at(1_usize).unwrap() = 1.0;

    let aResult = solve(&aA, &aB, 1e-15);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    assert_eq!(
        *aResult.solution.as_ref().unwrap().at(0_usize).unwrap(),
        1.0
    );
    assert_eq!(
        *aResult.solution.as_ref().unwrap().at(1_usize).unwrap(),
        1.0
    );
}
