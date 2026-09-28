#[test]
fn MathLin_Gauss_Test__SolveMultiple_DimensionMismatch() {
    let mut aA: Matrix = Matrix::filled(1, 3, 1, 3, 0.0).unwrap();
    for i in 1..=3 {
        aA[(i, i)] = 1.0;
    }

    let mut aBWrong: Matrix = Matrix::filled(1, 2, 1, 1, 0.0).unwrap();
    let aResult = solve_multiple(&aA, &aBWrong, 1e-15);
    assert_eq!(aResult.status, Status::InvalidInput);
}

#[test]
fn MathLin_Gauss_Test__ThreeByThreePivotedSystem() {
    let mut aMatrix: Matrix = Matrix::with_bounds(0, 3 - 1, 0, 3 - 1).unwrap();
    *aMatrix.change_at(0_usize, 0_usize).unwrap() = 4.0;
    *aMatrix.change_at(0_usize, 1_usize).unwrap() = 2.0;
    *aMatrix.change_at(0_usize, 2_usize).unwrap() = -1.0;
    *aMatrix.change_at(1_usize, 0_usize).unwrap() = 1.0;
    *aMatrix.change_at(1_usize, 1_usize).unwrap() = 5.0;
    *aMatrix.change_at(1_usize, 2_usize).unwrap() = 2.0;
    *aMatrix.change_at(2_usize, 0_usize).unwrap() = 2.0;
    *aMatrix.change_at(2_usize, 1_usize).unwrap() = -1.0;
    *aMatrix.change_at(2_usize, 2_usize).unwrap() = 3.0;

    let mut aRightHandSide: Vector = Vector::with_size(3);
    *aRightHandSide.change_at(0_usize).unwrap() = 7.0;
    *aRightHandSide.change_at(1_usize).unwrap() = -4.0;
    *aRightHandSide.change_at(2_usize).unwrap() = 5.0;
    let aResult = solve(&aMatrix, &aRightHandSide, 1e-15);

    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    let THE_EXPECTED_SOLUTION: [f64; 3] = [
        2.271_604_938_271_605,
        -1.1604938271604937,
        -0.23456790123456786,
    ];
    for anIndex in 0..3 {
        near(
            *aResult.solution.as_ref().unwrap().at(anIndex).unwrap(),
            THE_EXPECTED_SOLUTION[anIndex],
            1.0e-13,
        );
    }
}
