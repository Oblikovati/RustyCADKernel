#[test]
fn MathLin_ComparisonTest__Determinant_IdentityMatrix() {
    let mut aI: Matrix = Matrix::filled(1, 4, 1, 4, 0.0).unwrap();
    for i in 1..=4 {
        aI[(i, i)] = 1.0;
    }

    let aNewResult = determinant(&aI, 1e-15);
    assert!(aNewResult.is_done());

    let aOldGauss = Gauss::new(&aI).unwrap();
    assert!(aOldGauss.is_done());
    let aOldDet: f64 = aOldGauss.determinant().unwrap();

    near(aNewResult.determinant.unwrap(), 1.0, THE_TOLERANCE);
    near(aOldDet, 1.0, THE_TOLERANCE);
}

#[test]
fn MathLin_ComparisonTest__Invert_3x3Matrix() {
    let mut aA: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 2.0;
    aA[(1, 3)] = 3.0;
    aA[(2, 1)] = 0.0;
    aA[(2, 2)] = 1.0;
    aA[(2, 3)] = 4.0;
    aA[(3, 1)] = 5.0;
    aA[(3, 2)] = 6.0;
    aA[(3, 3)] = 0.0;

    let aNewResult = invert(&aA, 1e-15);
    assert!(aNewResult.is_done());
    assert!(aNewResult.inverse.is_some());

    let aOldGauss = Gauss::new(&aA).unwrap();
    assert!(aOldGauss.is_done());
    let mut aOldInv: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aOldGauss.invert(&mut aOldInv).unwrap();

    for i in 0..aNewResult.inverse.as_ref().unwrap().row_number() as usize {
        for j in 0..aNewResult.inverse.as_ref().unwrap().col_number() as usize {
            near(
                *aNewResult.inverse.as_ref().unwrap().at(i, j).unwrap(),
                *aOldInv.at(i, j).unwrap(),
                THE_TOLERANCE,
            );
        }
    }
}

#[test]
fn MathLin_ComparisonTest__Invert_VerifyByMultiplication() {
    let mut aA: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 2.0;
    aA[(1, 2)] = 1.0;
    aA[(1, 3)] = 1.0;
    aA[(2, 1)] = 1.0;
    aA[(2, 2)] = 3.0;
    aA[(2, 3)] = 2.0;
    aA[(3, 1)] = 1.0;
    aA[(3, 2)] = 2.0;
    aA[(3, 3)] = 2.0;

    let aNewResult = invert(&aA, 1e-15);
    assert!(aNewResult.is_done());

    let mut aProduct: Matrix = aA
        .multiplied_matrix(aNewResult.inverse.as_ref().unwrap())
        .unwrap();

    for i in 0..aProduct.row_number() as usize {
        for j in 0..aProduct.col_number() as usize {
            let aExpected: f64 = if i == j { 1.0 } else { 0.0 };
            near(*aProduct.at(i, j).unwrap(), aExpected, THE_TOLERANCE);
        }
    }
}
