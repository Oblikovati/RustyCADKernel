#[test]
fn MathLin_SVD_Test__WideMatrixReconstructionAndMinimumNorm() {
    let mut aA: Matrix = Matrix::filled(0, 2 - 1, 0, 3 - 1, 0.0).unwrap();
    *aA.change_at(0_usize, 0_usize).unwrap() = 1.0;
    *aA.change_at(0_usize, 2_usize).unwrap() = 1.0;
    *aA.change_at(1_usize, 1_usize).unwrap() = 1.0;
    *aA.change_at(1_usize, 2_usize).unwrap() = 1.0;

    let aSVD = svd(&aA, 1e-15);
    assert!(aSVD.is_done());
    assert_eq!(aSVD.u.as_ref().unwrap().row_number() as usize, 2);
    assert_eq!(aSVD.u.as_ref().unwrap().col_number() as usize, 3);
    assert_eq!(aSVD.rank, 2);
    for i in 0..aSVD.rank {
        let mut aNormSq: f64 = 0.0;
        for k in 0..aSVD.u.as_ref().unwrap().row_number() as usize {
            aNormSq += *aSVD.u.as_ref().unwrap().at(k, i).unwrap()
                * *aSVD.u.as_ref().unwrap().at(k, i).unwrap();
        }
        near(aNormSq, 1.0, 1.0e-12);
        for j in i + 1..aSVD.rank {
            let mut aDot: f64 = 0.0;
            for k in 0..aSVD.u.as_ref().unwrap().row_number() as usize {
                aDot += *aSVD.u.as_ref().unwrap().at(k, i).unwrap()
                    * *aSVD.u.as_ref().unwrap().at(k, j).unwrap();
            }
            near(aDot, 0.0, 1.0e-12);
        }
    }
    for i in 0..aA.row_number() as usize {
        for j in 0..aA.col_number() as usize {
            let mut aValue: f64 = 0.0;
            for k in 0..aSVD.singular_values.as_ref().unwrap().length() as usize {
                aValue += *aSVD.u.as_ref().unwrap().at(i, k).unwrap()
                    * *aSVD.singular_values.as_ref().unwrap().at(k).unwrap()
                    * *aSVD.v.as_ref().unwrap().at(j, k).unwrap();
            }
            near(aValue, *aA.at(i, j).unwrap(), 1.0e-12);
        }
    }

    let mut aB: Vector = Vector::filled(0, 2 - 1, 1.0).unwrap();
    let aSolve = solve_svd(&aA, &aB, 1e-06);
    assert!(aSolve.is_done());
    near(
        *aSolve.solution.as_ref().unwrap().at(0_usize).unwrap(),
        1.0 / 3.0,
        1.0e-12,
    );
    near(
        *aSolve.solution.as_ref().unwrap().at(1_usize).unwrap(),
        1.0 / 3.0,
        1.0e-12,
    );
    near(
        *aSolve.solution.as_ref().unwrap().at(2_usize).unwrap(),
        2.0 / 3.0,
        1.0e-12,
    );

    let aInverse = pseudo_inverse(&aA, 1e-06);
    assert!(aInverse.is_done());
    let aReconstructed: Matrix = matmul(&matmul(&aA, aInverse.inverse.as_ref().unwrap()), &aA);
    for i in 0..aA.row_number() as usize {
        for j in 0..aA.col_number() as usize {
            near(
                *aReconstructed.at(i, j).unwrap(),
                *aA.at(i, j).unwrap(),
                1.0e-12,
            );
        }
    }
}

#[test]
fn MathLin_Gauss_Test__SolveMultiple_ReturnsFullMatrix() {
    let mut aA: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 4.0;
    aA[(1, 2)] = 1.0;
    aA[(1, 3)] = 2.0;
    aA[(2, 1)] = 0.0;
    aA[(2, 2)] = 3.0;
    aA[(2, 3)] = 1.0;
    aA[(3, 1)] = 2.0;
    aA[(3, 2)] = 1.0;
    aA[(3, 3)] = 5.0;

    let mut aXExpected: Matrix = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    aXExpected[(1, 1)] = 1.0;
    aXExpected[(2, 1)] = 2.0;
    aXExpected[(3, 1)] = -1.0;
    aXExpected[(1, 2)] = -2.0;
    aXExpected[(2, 2)] = 0.5;
    aXExpected[(3, 2)] = 3.0;

    let aB: Matrix = matmul(&aA, &aXExpected);

    let aResult = solve_multiple(&aA, &aB, 1e-15);
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
