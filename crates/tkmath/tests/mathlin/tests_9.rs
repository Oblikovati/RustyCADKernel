#[test]
fn MathLin_Gauss_Test__ThreeByThreeDeterminantAndInverse() {
    let THE_A: [[f64; 3]; 3] = [[4.0, -1.0, 2.0], [0.5, 3.0, -0.75], [-2.0, 1.0, 5.0]];
    let THE_EXPECTED_INVERSE: [[f64; 3]; 3] = [
        [
            0.20454545454545453,
            0.090_909_090_909_090_91,
            -0.068_181_818_181_818_18,
        ],
        [
            -0.012987012987012986,
            0.311_688_311_688_311_7,
            0.051948051948051945,
        ],
        [
            0.084_415_584_415_584_42,
            -0.025974025974025972,
            0.16233766233766234,
        ],
    ];
    let mut aA: Matrix = Matrix::with_bounds(0, 3 - 1, 0, 3 - 1).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            *aA.change_at(i, j).unwrap() = THE_A[i][j];
        }
    }

    let aResult = invert(&aA, 1e-15);
    assert!(aResult.is_done());
    assert!(aResult.inverse.is_some());
    assert!(aResult.determinant.is_some());
    near(aResult.determinant.unwrap(), 77.0, 1.0e-13);

    for i in 0..3 {
        for j in 0..3 {
            near(
                *aResult.inverse.as_ref().unwrap().at(i, j).unwrap(),
                THE_EXPECTED_INVERSE[i][j],
                1.0e-13,
            );
            let mut aResidual: f64 = if i == j { -1.0 } else { 0.0 };
            for k in 0..3 {
                aResidual += *aA.at(i, (k) as usize).unwrap()
                    * *aResult
                        .inverse
                        .as_ref()
                        .unwrap()
                        .at((k) as usize, j)
                        .unwrap();
            }
            near(aResidual, 0.0, 1.0e-13);
        }
    }
}

#[test]
fn MathLin_Crout_Test__MismatchedBoundsFactorAndSolve() {
    let mut aA: Matrix = Matrix::with_bounds(-1, 1, 4, 6).unwrap();
    let aValues: [[f64; 3]; 3] = [[4.0, 1.0, 1.0], [1.0, 3.0, 0.5], [1.0, 0.5, 2.0]];
    for i in 0..3 {
        for j in 0..3 {
            aA[(-1 + i, 4 + j)] = aValues[i as usize][j as usize];
        }
    }
    let mut aB: Vector = Vector::with_bounds(10, 12).unwrap();
    for i in 0..3 {
        aB[10 + i] = aValues[i as usize][0_usize] + 2.0 * aValues[i as usize][1_usize]
            - aValues[i as usize][2_usize];
    }
    let aFactor = crout(&aA, 1e-15);
    assert!(aFactor.is_done());
    for i in 0..3 {
        for j in 0..3 {
            let mut aValue: f64 = 0.0;
            for k in 0..3 {
                let mut aLik: f64 = if i >= k {
                    *aFactor
                        .lower
                        .as_ref()
                        .unwrap()
                        .at((i) as usize, (k) as usize)
                        .unwrap()
                } else {
                    0.0
                };
                let mut aLjk: f64 = if j >= k {
                    *aFactor
                        .lower
                        .as_ref()
                        .unwrap()
                        .at((j) as usize, (k) as usize)
                        .unwrap()
                } else {
                    0.0
                };
                aValue +=
                    aLik * *aFactor.diagonal.as_ref().unwrap().at((k) as usize).unwrap() * aLjk;
            }
            near(aValue, aValues[i as usize][j as usize], 1.0e-12);
        }
    }
    let aSolve = solve_crout(&aA, &aB, 1e-15);
    assert!(aSolve.is_done());
    assert_eq!(aSolve.solution.as_ref().unwrap().lower(), 0);
    near(
        *aSolve.solution.as_ref().unwrap().at(0_usize).unwrap(),
        1.0,
        1.0e-12,
    );
    near(
        *aSolve.solution.as_ref().unwrap().at(1_usize).unwrap(),
        2.0,
        1.0e-12,
    );
    near(
        *aSolve.solution.as_ref().unwrap().at(2_usize).unwrap(),
        -1.0,
        1.0e-12,
    );
}
