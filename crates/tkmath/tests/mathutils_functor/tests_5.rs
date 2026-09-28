#[test]
fn MathUtils_Functor_Vector__Himmelblau_Value() {
    let mut aHimmel = Himmelblau;

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 3.0;
    aX[2] = 2.0;

    let mut aValue: f64 = 0.0;
    assert!(aHimmel.value(&aX, &mut aValue));
    near(aValue, 0.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__Rastrigin_Value() {
    let mut aRast = Rastrigin::new(2, 10.0);

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 0.0;
    aX[2] = 0.0;

    let mut aValue: f64 = 0.0;
    assert!(aRast.value(&aX, &mut aValue));
    near(aValue, 0.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__Ackley_Value() {
    let mut aAckley = Ackley::new(2, 20.0, 0.2, TWO_PI);

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 0.0;
    aX[2] = 0.0;

    let mut aValue: f64 = 0.0;
    assert!(aAckley.value(&aX, &mut aValue));
    near(aValue, 0.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__AckleyRejectsEmptyAndNonFiniteInput() {
    let mut aAckley = Ackley::new(2, 20.0, 0.2, TWO_PI);
    let mut aValue: f64 = 0.0;

    let mut anEmpty = Vector::with_size(0);
    assert!(!(aAckley.value(&anEmpty, &mut aValue)));

    let mut aNonFinite = Vector::filled(0, 1 - 1, f64::INFINITY).unwrap();
    assert!(!(aAckley.value(&aNonFinite, &mut aValue)));

    let mut anInvalid = Ackley::new(2, f64::INFINITY, 0.2, TWO_PI);
    let mut anOrigin = Vector::filled(0, 1 - 1, 0.0).unwrap();
    assert!(!(anInvalid.value(&anOrigin, &mut aValue)));
}

#[test]
fn MathUtils_Functor_Vector__QuadraticForm_Value() {
    let mut aA = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(2, 2)] = 1.0;

    let mut aB = Vector::filled(1, 2, 0.0).unwrap();
    let mut aC: f64 = 0.0;

    let mut aQuad = QuadraticForm::new(&aA, &aB, aC);

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 3.0;
    aX[2] = 4.0;

    let mut aValue: f64 = 0.0;
    assert!(aQuad.value(&aX, &mut aValue));
    near(aValue, 25.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__QuadraticForm_Gradient() {
    let mut aA = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(2, 2)] = 1.0;

    let mut aB = Vector::filled(1, 2, 0.0).unwrap();
    let mut aC: f64 = 0.0;

    let mut aQuad = QuadraticForm::new(&aA, &aB, aC);

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 3.0;
    aX[2] = 4.0;

    let mut aGrad = Vector::with_bounds(1, 2).unwrap();
    assert!(aQuad.gradient(&aX, &mut aGrad));
    near(aGrad[1], 6.0, THE_TOLERANCE);
    near(aGrad[2], 8.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__LinearResidual_Value() {
    let mut aA = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(2, 2)] = 1.0;

    let mut aB = Vector::with_bounds(1, 2).unwrap();
    aB[1] = 3.0;
    aB[2] = 4.0;

    let mut aRes = LinearResidual::new(&aA, &aB);

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 3.0;
    aX[2] = 4.0;

    let mut aValue: f64 = 0.0;
    assert!(aRes.value(&aX, &mut aValue));
    near(aValue, 0.0, THE_TOLERANCE);
}
