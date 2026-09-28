#[test]
fn MathLin_SVD_Test__PseudoInverseUsesEffectiveSVDCutoff() {
    let mut aA: Matrix = Matrix::filled(0, 2 - 1, 0, 2 - 1, 0.0).unwrap();
    *aA.change_at(0_usize, 0_usize).unwrap() = 1.0;
    *aA.change_at(1_usize, 1_usize).unwrap() = f64::EPSILON;

    let aSVD = svd(&aA, 0.0);
    assert!(aSVD.is_done());
    assert_eq!(aSVD.rank, 1);

    let mut aB: Vector = Vector::filled(0, 2 - 1, 1.0).unwrap();
    let aSolve = solve_svd(&aA, &aB, 0.0);
    let aInverse = pseudo_inverse(&aA, 0.0);
    assert!(aSolve.is_done());
    assert!(aInverse.is_done());
    assert_eq!(*aSolve.solution.as_ref().unwrap().at(1_usize).unwrap(), 0.0);
    assert_eq!(
        *aInverse
            .inverse
            .as_ref()
            .unwrap()
            .at(1_usize, 1_usize)
            .unwrap(),
        0.0
    );
}

#[test]
fn MathLin_SVD_Test__PseudoInverseRejectsNonFiniteOutput() {
    let mut aA: Matrix = Matrix::filled(0, 2 - 1, 0, 2 - 1, 0.0).unwrap();
    *aA.change_at(0_usize, 0_usize).unwrap() = 1.0e-307;
    *aA.change_at(1_usize, 1_usize).unwrap() = 1.0e-309;

    assert_eq!(pseudo_inverse(&aA, 0.0).status, Status::NumericalError);
}
