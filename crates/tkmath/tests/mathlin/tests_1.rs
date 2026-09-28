#[test]
fn MathLin_SVD_Test__SolveSystem() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMat[(1, 1)] = 3.0;
    aMat[(1, 2)] = 1.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 2.0;

    let mut aB: Vector = Vector::with_bounds(1, 2).unwrap();
    aB[1] = 9.0;
    aB[2] = 8.0;

    let aResult = solve_svd(&aMat, &aB, 1e-06);

    assert!(aResult.is_done());

    let aX: &Vector = aResult.solution.as_ref().unwrap();
    let mut aCheck1: f64 =
        aMat[(1, 1)] * *aX.at(0_usize).unwrap() + aMat[(1, 2)] * *aX.at(1_usize).unwrap();
    let mut aCheck2: f64 =
        aMat[(2, 1)] * *aX.at(0_usize).unwrap() + aMat[(2, 2)] * *aX.at(1_usize).unwrap();

    near(aCheck1, aB[1], THE_TOLERANCE);
    near(aCheck2, aB[2], THE_TOLERANCE);
}

#[test]
fn MathLin_SVD_Test__PseudoInverse() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 2.0;
    aMat[(2, 1)] = 3.0;
    aMat[(2, 2)] = 4.0;

    let aPinv = pseudo_inverse(&aMat, 1e-06);

    assert!(aPinv.is_done());

    let aTemp: Matrix = matmul(&aMat, aPinv.inverse.as_ref().unwrap());
    let aCheck: Matrix = matmul(&aTemp, &aMat);

    for i in 1..=2 {
        for j in 1..=2 {
            near(aCheck[(i, j)], aMat[(i, j)], THE_TOLERANCE);
        }
    }
}
