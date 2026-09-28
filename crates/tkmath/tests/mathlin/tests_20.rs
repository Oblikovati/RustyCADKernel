#[test]
fn MathLin_LeastSquares_Test__WeightedLeastSquares() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 1.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 2.0;
    aMat[(3, 1)] = 1.0;
    aMat[(3, 2)] = 3.0;

    let mut aB: Vector = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 2.0;
    aB[2] = 3.0;
    aB[3] = 4.5;

    let mut aW1: Vector = Vector::filled(1, 3, 1.0).unwrap();
    let aResult1 = weighted_least_squares(&aMat, &aB, &aW1, LeastSquaresMethod::Qr, 1e-15);

    let mut aW2: Vector = Vector::with_bounds(1, 3).unwrap();
    aW2[1] = 10.0;
    aW2[2] = 10.0;
    aW2[3] = 0.1;
    let aResult2 = weighted_least_squares(&aMat, &aB, &aW2, LeastSquaresMethod::Qr, 1e-15);

    assert!(aResult1.is_done());
    assert!(aResult2.is_done());

    assert_ne!(
        *aResult1.solution.as_ref().unwrap().at(0_usize).unwrap(),
        *aResult2.solution.as_ref().unwrap().at(0_usize).unwrap()
    );
}

#[test]
fn MathLin_LeastSquares_Test__RegularizedLeastSquares() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            aMat[(i, j)] = 1.0 / ((i + j - 1) as f64);
        }
    }

    let mut aB: Vector = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 1.0;
    aB[2] = 0.5;
    aB[3] = 0.333;

    let aResultNoReg = least_squares(&aMat, &aB, LeastSquaresMethod::Qr, 1e-15);
    let aResultReg = regularized_least_squares(&aMat, &aB, 0.01, 1e-15);

    assert!(aResultNoReg.is_done());
    assert!(aResultReg.is_done());

    let mut aNormNoReg: f64 = vectornorm(aResultNoReg.solution.as_ref().unwrap());
    let mut aNormReg: f64 = vectornorm(aResultReg.solution.as_ref().unwrap());
    assert!(aNormReg < aNormNoReg);
}
