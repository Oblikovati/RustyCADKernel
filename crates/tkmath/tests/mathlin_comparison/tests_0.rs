#[test]
fn MathLin_ComparisonTest__Solve_3x3System() {
    let mut aA: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 3.0;
    aA[(1, 2)] = 2.0;
    aA[(1, 3)] = -1.0;
    aA[(2, 1)] = 2.0;
    aA[(2, 2)] = -2.0;
    aA[(2, 3)] = 4.0;
    aA[(3, 1)] = -1.0;
    aA[(3, 2)] = 0.5;
    aA[(3, 3)] = -1.0;

    let mut aB: Vector = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 1.0;
    aB[2] = -2.0;
    aB[3] = 0.0;

    let aNewResult = solve(&aA, &aB, 1e-15);
    assert!(aNewResult.is_done());
    assert!(aNewResult.solution.is_some());

    let aOldGauss = Gauss::new(&aA).unwrap();
    assert!(aOldGauss.is_done());
    let mut aOldX: Vector = Vector::with_bounds(1, 3).unwrap();
    aOldGauss.solve_into(&aB, &mut aOldX).unwrap();

    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            *aOldX.at(i).unwrap(),
            THE_TOLERANCE,
        );
    }
}

#[test]
fn MathLin_ComparisonTest__Solve_5x5System() {
    let mut aA: Matrix = Matrix::filled(1, 5, 1, 5, 0.0).unwrap();
    for i in 1..=5 {
        aA[(i, i)] = 10.0;
        if i > 1 {
            aA[(i, i - 1)] = -2.0;
        }
        if i < 5 {
            aA[(i, i + 1)] = -3.0;
        }
    }

    let mut aB: Vector = Vector::with_bounds(1, 5).unwrap();
    for i in 1..=5 {
        aB[i] = (i) as f64;
    }

    let aNewResult = solve(&aA, &aB, 1e-15);
    assert!(aNewResult.is_done());

    let aOldGauss = Gauss::new(&aA).unwrap();
    assert!(aOldGauss.is_done());
    let mut aOldX: Vector = Vector::with_bounds(1, 5).unwrap();
    aOldGauss.solve_into(&aB, &mut aOldX).unwrap();

    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            *aOldX.at(i).unwrap(),
            THE_TOLERANCE,
        );
    }
}

#[test]
fn MathLin_ComparisonTest__Determinant_3x3Matrix() {
    let mut aA: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 6.0;
    aA[(1, 2)] = 1.0;
    aA[(1, 3)] = 1.0;
    aA[(2, 1)] = 4.0;
    aA[(2, 2)] = -2.0;
    aA[(2, 3)] = 5.0;
    aA[(3, 1)] = 2.0;
    aA[(3, 2)] = 8.0;
    aA[(3, 3)] = 7.0;

    let aNewResult = determinant(&aA, 1e-15);
    assert!(aNewResult.is_done());
    assert!(aNewResult.determinant.is_some());

    let aOldGauss = Gauss::new(&aA).unwrap();
    assert!(aOldGauss.is_done());
    let aOldDet: f64 = aOldGauss.determinant().unwrap();

    near(aNewResult.determinant.unwrap(), aOldDet, THE_TOLERANCE);
    near(aNewResult.determinant.unwrap(), -306.0, THE_TOLERANCE);
}
