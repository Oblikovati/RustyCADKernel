#[test]
fn MathLin_SVD_Test__ConditionNumber() {
    let aI: Matrix = CreateIdentity(3);
    let aCondI: Option<f64> = condition_number(&aI);
    assert!(aCondI.is_some());
    near(aCondI.unwrap(), 1.0, THE_TOLERANCE);

    let mut aHilbert: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            aHilbert[(i, j)] = 1.0 / ((i + j - 1) as f64);
        }
    }
    let aCondH: Option<f64> = condition_number(&aHilbert);
    assert!(aCondH.is_some());
    assert!(aCondH.unwrap() > 100.0);
}

#[test]
fn MathLin_SVD_Test__RankDeficiencyBoundsAndSortedReconstruction() {
    let mut aA: Matrix = Matrix::with_bounds(-2, 0, 7, 8).unwrap();
    aA[(-2, 7)] = 1.0;
    aA[(-2, 8)] = 2.0;
    aA[(-1, 7)] = 2.0;
    aA[(-1, 8)] = 4.0;
    aA[(0, 7)] = -1.0;
    aA[(0, 8)] = -2.0;

    let aSVD = svd(&aA, 1e-15);
    assert!(aSVD.is_done());
    assert_eq!(aSVD.rank, 1);
    assert!(!(aSVD.condition_number.is_some()));
    assert!(
        *aSVD.singular_values.as_ref().unwrap().at(0_usize).unwrap()
            >= *aSVD.singular_values.as_ref().unwrap().at(1_usize).unwrap()
    );
    for i in 0..aSVD.u.as_ref().unwrap().row_number() as usize {
        for j in 0..aSVD.v.as_ref().unwrap().row_number() as usize {
            let mut aValue: f64 = 0.0;
            for k in 0..aSVD.singular_values.as_ref().unwrap().length() as usize {
                aValue += *aSVD.u.as_ref().unwrap().at(i, k).unwrap()
                    * *aSVD.singular_values.as_ref().unwrap().at(k).unwrap()
                    * *aSVD.v.as_ref().unwrap().at(j, k).unwrap();
            }
            near(aValue, aA[(-2 + (i) as i32, 7 + (j) as i32)], 1.0e-12);
        }
    }
}
