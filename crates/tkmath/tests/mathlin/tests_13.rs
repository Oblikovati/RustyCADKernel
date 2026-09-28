#[test]
fn MathLin_Householder_Test__Overdetermined() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 1.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 2.0;
    aMat[(3, 1)] = 1.0;
    aMat[(3, 2)] = 3.0;

    let mut aB: Vector = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 1.0;
    aB[2] = 2.0;
    aB[3] = 3.0;

    let aResult = solve_qr(&aMat, &aB, 1e-15);

    assert!(aResult.is_done());

    assert_eq!(aResult.solution.as_ref().unwrap().length() as usize, 2);
}

#[test]
fn MathLin_Householder_Test__BoundsScalingRankAndReconstruction() {
    let mut aA: Matrix = Matrix::with_bounds(-4, -2, 3, 4).unwrap();
    let mut aScale: f64 = 1.0e150;
    aA[(-4, 3)] = 1.0 * aScale;
    aA[(-4, 4)] = 2.0 * aScale;
    aA[(-3, 3)] = 2.0 * aScale;
    aA[(-3, 4)] = -aScale;
    aA[(-2, 3)] = 3.0 * aScale;
    aA[(-2, 4)] = 1.0 * aScale;
    let aQR = qr(&aA, 1e-15);
    assert!(aQR.is_done());
    assert_eq!(aQR.rank, 2);
    assert!(aQR.condition_estimate.is_some());
    for i in 0..aQR.q.as_ref().unwrap().row_number() as usize {
        for j in 0..aQR.r.as_ref().unwrap().col_number() as usize {
            let mut aValue: f64 = 0.0;
            for k in 0..aQR.q.as_ref().unwrap().col_number() as usize {
                aValue += *aQR.q.as_ref().unwrap().at(i, k).unwrap()
                    * *aQR.r.as_ref().unwrap().at(k, j).unwrap();
            }
            near(
                aValue / aScale,
                aA[(-4 + (i) as i32, 3 + (j) as i32)] / aScale,
                1.0e-12,
            );
        }
    }

    aA[(-4, 4)] = 2.0 * aA[(-4, 3)];
    aA[(-3, 4)] = 2.0 * aA[(-3, 3)];
    aA[(-2, 4)] = 2.0 * aA[(-2, 3)];
    let aRankDeficientQR = qr(&aA, 1e-15);
    assert_eq!(aRankDeficientQR.rank, 1);
    assert!(!(aRankDeficientQR.condition_estimate.is_some()));
    let mut aB: Vector = Vector::filled(10, 12, 1.0).unwrap();
    assert_eq!(solve_qr(&aA, &aB, 1e-15).status, Status::Singular);
}
