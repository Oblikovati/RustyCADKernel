#[test]
fn MathLin_ComparisonTest__Determinant_MultipleMatrices() {
    for aCase in [
        [1.0, 0.0, 0.0, 1.0, 1.0],
        [2.0, 0.0, 0.0, 3.0, 6.0],
        [1.0, 2.0, 3.0, 4.0, -2.0],
        [0.0, 1.0, 1.0, 0.0, -1.0],
    ] {
        let mut aA: Matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
        aA[(1, 1)] = aCase[0];
        aA[(1, 2)] = aCase[1];
        aA[(2, 1)] = aCase[2];
        aA[(2, 2)] = aCase[3];

        let aNewResult = determinant(&aA, 1e-15);
        assert!(aNewResult.is_done());

        let aOldGauss = Gauss::new(&aA).unwrap();
        assert!(aOldGauss.is_done());

        near(
            aNewResult.determinant.unwrap(),
            aOldGauss.determinant().unwrap(),
            THE_TOLERANCE,
        );
        near(aNewResult.determinant.unwrap(), aCase[4], THE_TOLERANCE);
    }
}

#[test]
fn MathLin_ComparisonTest__Solve_KnownSolution() {
    let mut aA: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 0.0;
    aA[(1, 3)] = 0.0;
    aA[(2, 1)] = 0.0;
    aA[(2, 2)] = 1.0;
    aA[(2, 3)] = 0.0;
    aA[(3, 1)] = 0.0;
    aA[(3, 2)] = 0.0;
    aA[(3, 3)] = 1.0;

    let mut aExpectedX: Vector = Vector::with_bounds(1, 3).unwrap();
    aExpectedX[1] = 1.0;
    aExpectedX[2] = 2.0;
    aExpectedX[3] = 3.0;

    let mut aB: Vector = aA.multiplied_vector(&aExpectedX).unwrap();

    let aNewResult = solve(&aA, &aB, 1e-15);
    assert!(aNewResult.is_done());

    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            *aExpectedX.at(i).unwrap(),
            THE_TOLERANCE,
        );
    }
}

#[test]
fn MathLin_ComparisonTest__Solve_ComplexKnownSolution() {
    let mut aA: Matrix = Matrix::with_bounds(1, 4, 1, 4).unwrap();
    aA[(1, 1)] = 5.0;
    aA[(1, 2)] = 7.0;
    aA[(1, 3)] = 6.0;
    aA[(1, 4)] = 5.0;
    aA[(2, 1)] = 7.0;
    aA[(2, 2)] = 10.0;
    aA[(2, 3)] = 8.0;
    aA[(2, 4)] = 7.0;
    aA[(3, 1)] = 6.0;
    aA[(3, 2)] = 8.0;
    aA[(3, 3)] = 10.0;
    aA[(3, 4)] = 9.0;
    aA[(4, 1)] = 5.0;
    aA[(4, 2)] = 7.0;
    aA[(4, 3)] = 9.0;
    aA[(4, 4)] = 10.0;

    let mut aExpectedX: Vector = Vector::with_bounds(1, 4).unwrap();
    aExpectedX[1] = 1.0;
    aExpectedX[2] = -1.0;
    aExpectedX[3] = 2.0;
    aExpectedX[4] = -2.0;

    let mut aB: Vector = aA.multiplied_vector(&aExpectedX).unwrap();

    let aNewResult = solve(&aA, &aB, 1e-15);
    assert!(aNewResult.is_done());

    let aOldGauss = Gauss::new(&aA).unwrap();
    assert!(aOldGauss.is_done());
    let mut aOldX: Vector = Vector::with_bounds(1, 4).unwrap();
    aOldGauss.solve_into(&aB, &mut aOldX).unwrap();

    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            *aExpectedX.at(i).unwrap(),
            THE_TOLERANCE,
        );
        near(
            *aOldX.at(i).unwrap(),
            *aExpectedX.at(i).unwrap(),
            THE_TOLERANCE,
        );
        near(
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            *aOldX.at(i).unwrap(),
            THE_TOLERANCE,
        );
    }
}
