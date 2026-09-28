#[test]
fn MathLin_Householder_Test__BasicQR_2x2() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 2.0;
    aMat[(2, 1)] = 3.0;
    aMat[(2, 2)] = 4.0;

    let aResult = qr(&aMat, 1e-15);

    assert!(aResult.is_done());
    assert!(aResult.condition_estimate.is_some());
    assert!(aResult.condition_estimate.unwrap().is_finite());

    let aQ: &Matrix = aResult.q.as_ref().unwrap();
    let aR: &Matrix = aResult.r.as_ref().unwrap();

    let aQQt: Matrix = matmul(aQ, &transpose(aQ));
    for i in 1..=2 {
        for j in 1..=2 {
            let mut aExpected: f64 = if i == j { 1.0 } else { 0.0 };
            near(aQQt[(i, j)], aExpected, THE_TOLERANCE);
        }
    }

    near(*aR.at(1_usize, 0_usize).unwrap(), 0.0, THE_TOLERANCE);

    let aQR: Matrix = matmul(aQ, aR);
    for i in 1..=2 {
        for j in 1..=2 {
            near(aQR[(i, j)], aMat[(i, j)], THE_TOLERANCE);
        }
    }
}

#[test]
fn MathLin_Householder_Test__SolveSystem() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMat[(1, 1)] = 3.0;
    aMat[(1, 2)] = 1.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 2.0;

    let mut aB: Vector = Vector::with_bounds(1, 2).unwrap();
    aB[1] = 9.0;
    aB[2] = 8.0;

    let aResult = solve_qr(&aMat, &aB, 1e-15);

    assert!(aResult.is_done());

    let aX: &Vector = aResult.solution.as_ref().unwrap();
    let mut aCheck1: f64 =
        aMat[(1, 1)] * *aX.at(0_usize).unwrap() + aMat[(1, 2)] * *aX.at(1_usize).unwrap();
    let mut aCheck2: f64 =
        aMat[(2, 1)] * *aX.at(0_usize).unwrap() + aMat[(2, 2)] * *aX.at(1_usize).unwrap();

    near(aCheck1, aB[1], THE_TOLERANCE);
    near(aCheck2, aB[2], THE_TOLERANCE);
}
