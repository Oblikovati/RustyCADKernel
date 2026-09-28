#[test]
fn MathLin_Gauss_Test__FiveByFiveColumnScaledSystem() {
    let THE_A: [[f64; 5]; 5] = [
        [2.5, 0.0012, -3.0, 1.5, 0.2],
        [-1.0, 0.0021, 2.0, 0.5, -0.7],
        [3.0, -0.0008, 1.0, -2.0, 0.4],
        [0.5, 0.0032, -1.0, 4.0, 1.1],
        [-2.0, 0.0017, 0.5, 1.0, 3.0],
    ];
    let THE_B: [f64; 5] = [1.2, -3.4, 5.6, 0.75, -2.1];
    let mut aA: Matrix = Matrix::with_bounds(0, 5 - 1, 0, 5 - 1).unwrap();
    let mut aB: Vector = Vector::with_size(5usize);
    for i in 0..5 {
        *aB.change_at(i).unwrap() = THE_B[i];
        for j in 0..5 {
            *aA.change_at(i, j).unwrap() = THE_A[i][j];
        }
    }

    let aResult = solve(&aA, &aB, 1e-15);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    assert!(aResult.determinant.is_some());

    let THE_EXPECTED_SOLUTION: [f64; 5] = [
        1.8626125409601375,
        -1955.7312378710285,
        1.2160500111347952,
        1.5690929914421161,
        0.924_283_396_430_503_2,
    ];
    for i in 0..5 {
        near(
            *aResult.solution.as_ref().unwrap().at(i).unwrap(),
            THE_EXPECTED_SOLUTION[i],
            1.0e-9,
        );
        let mut aResidual: f64 = -*aB.at(i).unwrap();
        for j in 0..5 {
            aResidual += *aA.at(i, (j) as usize).unwrap()
                * *aResult.solution.as_ref().unwrap().at((j) as usize).unwrap();
        }
        near(aResidual, 0.0, 1.0e-11);
    }
    near(aResult.determinant.unwrap(), 0.23574749999999986, 1.0e-13);
}

#[test]
fn MathLin_Gauss_Test__ArbitraryMismatchedBoundsAndScaledSystem() {
    let mut aA: Matrix = Matrix::with_bounds(-3, -1, 5, 7).unwrap();
    let mut aScale: f64 = 1.0e-200;
    aA[(-3, 5)] = 4.0 * aScale;
    aA[(-3, 6)] = 1.0 * aScale;
    aA[(-3, 7)] = 2.0 * aScale;
    aA[(-2, 5)] = 1.0 * aScale;
    aA[(-2, 6)] = 5.0 * aScale;
    aA[(-2, 7)] = 1.0 * aScale;
    aA[(-1, 5)] = 2.0 * aScale;
    aA[(-1, 6)] = 1.0 * aScale;
    aA[(-1, 7)] = 6.0 * aScale;
    let mut aB: Vector = Vector::with_bounds(20, 22).unwrap();
    aB[20] = 12.0 * aScale;
    aB[21] = 14.0 * aScale;
    aB[22] = 22.0 * aScale;

    let aSolve = solve(&aA, &aB, 1e-15);
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
        3.0,
        1.0e-12,
    );

    let aFactor = lu(&aA, 1e-15);
    assert!(aFactor.is_done());
    let mut aPermuted: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            aPermuted[(i, j)] = aA[(-4 + i, 4 + j)];
        }
    }
    for k in 0..aFactor.pivot.as_ref().unwrap().len() as usize {
        let aPivot: usize = *aFactor.pivot.as_ref().unwrap().at(k).unwrap() as usize;
        for j in 0..aFactor.lu.as_ref().unwrap().col_number() as usize {
            swap_entries(&mut aPermuted, k, aPivot, j);
        }
    }
    let mut aL: Matrix = Matrix::filled(1, 3, 1, 3, 0.0).unwrap();
    let mut aU: Matrix = Matrix::filled(1, 3, 1, 3, 0.0).unwrap();
    for i in 1..=3 {
        aL[(i, i)] = 1.0;
        for j in 1..=3 {
            if i > j {
                aL[(i, j)] = *aFactor
                    .lu
                    .as_ref()
                    .unwrap()
                    .at((i - 1) as usize, (j - 1) as usize)
                    .unwrap();
            } else {
                aU[(i, j)] = *aFactor
                    .lu
                    .as_ref()
                    .unwrap()
                    .at((i - 1) as usize, (j - 1) as usize)
                    .unwrap();
            }
        }
    }
    let aReconstructed: Matrix = aL.multiplied_matrix(&aU).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            near(
                aReconstructed[(i, j)] / aScale,
                aPermuted[(i, j)] / aScale,
                1.0e-12,
            );
        }
    }
}
