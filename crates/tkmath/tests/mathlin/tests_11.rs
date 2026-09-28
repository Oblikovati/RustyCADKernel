#[test]
fn MathLin_Householder_Test__SolveQRMultiple_ReturnsFullMatrix() {
    let mut aA: Matrix = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 2.0;
    aA[(2, 1)] = 3.0;
    aA[(2, 2)] = 1.0;
    aA[(3, 1)] = -1.0;
    aA[(3, 2)] = 1.0;

    let mut aXExpected: Matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aXExpected[(1, 1)] = 2.0;
    aXExpected[(2, 1)] = -1.0;
    aXExpected[(1, 2)] = -0.5;
    aXExpected[(2, 2)] = 3.0;

    let aB: Matrix = matmul(&aA, &aXExpected);

    let aResult = solve_qr_multiple(&aA, &aB, 1e-15);
    assert!(aResult.is_done());
    assert!(aResult.solutions.is_some());

    let aX: &Matrix = aResult.solutions.as_ref().unwrap();
    for i in 0..aX.row_number() as usize {
        for j in 0..aX.col_number() as usize {
            near(
                *aX.at(i, j).unwrap(),
                *aXExpected.at(i, j).unwrap(),
                THE_TOLERANCE,
            );
        }
    }

    let aCheckB: Matrix = matmul(&aA, aX);
    for i in 1..=3 {
        for j in 1..=2 {
            near(aCheckB[(i, j)], aB[(i, j)], THE_TOLERANCE);
        }
    }
}

#[test]
fn MathLin_Householder_Test__SolveQRMultiple_DimensionMismatch() {
    let mut aA: Matrix = Matrix::filled(1, 3, 1, 2, 0.0).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(2, 2)] = 1.0;

    let mut aBWrong: Matrix = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    let aResult = solve_qr_multiple(&aA, &aBWrong, 1e-15);
    assert_eq!(aResult.status, Status::InvalidInput);
}
