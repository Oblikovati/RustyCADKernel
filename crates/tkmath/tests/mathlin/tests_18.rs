#[test]
fn MathLin_LeastSquares_Test__SquareSystem() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMat[(1, 1)] = 3.0;
    aMat[(1, 2)] = 1.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 2.0;

    let mut aB: Vector = Vector::with_bounds(1, 2).unwrap();
    aB[1] = 9.0;
    aB[2] = 8.0;

    let aResult = least_squares(&aMat, &aB, LeastSquaresMethod::Qr, 1e-15);

    assert!(aResult.is_done());

    assert!(aResult.residual.unwrap() < THE_TOLERANCE);
}

#[test]
fn MathLin_LeastSquares_Test__Overdetermined() {
    let mut aMat: Matrix = Matrix::with_bounds(1, 4, 1, 2).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 1.0;
    aMat[(2, 1)] = 1.0;
    aMat[(2, 2)] = 2.0;
    aMat[(3, 1)] = 1.0;
    aMat[(3, 2)] = 3.0;
    aMat[(4, 1)] = 1.0;
    aMat[(4, 2)] = 4.0;

    let mut aB: Vector = Vector::with_bounds(1, 4).unwrap();
    aB[1] = 2.0;
    aB[2] = 3.0;
    aB[3] = 4.0;
    aB[4] = 5.0;

    let aResult = least_squares(&aMat, &aB, LeastSquaresMethod::Qr, 1e-15);

    assert!(aResult.is_done());

    let aX: &Vector = aResult.solution.as_ref().unwrap();

    near(*aX.at(0_usize).unwrap(), 1.0, THE_TOLERANCE);
    near(*aX.at(1_usize).unwrap(), 1.0, THE_TOLERANCE);

    assert!(aResult.residual.unwrap() < THE_TOLERANCE);
}
