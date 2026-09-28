#[test]
fn MathUtils_Functor_Scalar__Exponential_Value() {
    let mut aExp = Exponential::new(1.0, 1.0, 0.0);

    let mut aValue: f64 = 0.0;
    assert!(aExp.value(0.0, &mut aValue));
    near(aValue, 1.0, THE_TOLERANCE);

    assert!(aExp.value(1.0, &mut aValue));
    near(aValue, (1.0_f64).exp(), THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Power_Value() {
    let mut aPower = Power::new(2.0, 1.0, 0.0);

    let mut aValue: f64 = 0.0;
    assert!(aPower.value(3.0, &mut aValue));
    near(aValue, 9.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Power_NonInteger() {
    let mut aPower = Power::new(0.5, 1.0, 0.0);

    let mut aValue: f64 = 0.0;
    assert!(aPower.value(4.0, &mut aValue));
    near(aValue, 2.0, THE_TOLERANCE);

    assert!(!(aPower.value(-4.0, &mut aValue)));
}

#[test]
fn MathUtils_Functor_Scalar__PowerZeroAtZeroHasZeroDerivative() {
    let mut aPower = Power::new(0.0, 2.0, 3.0);
    let mut aValue: f64 = 0.0;
    let mut aDerivative: f64 = 1.0;

    assert!(aPower.values(0.0, &mut aValue, &mut aDerivative));
    assert_eq!(aValue, 5.0);
    assert_eq!(aDerivative, 0.0);
}

#[test]
fn MathUtils_Functor_Scalar__Gaussian_Value() {
    let mut aGauss = Gaussian::new(1.0, 0.0, 1.0);

    let mut aValue: f64 = 0.0;
    assert!(aGauss.value(0.0, &mut aValue));
    near(aValue, 1.0, THE_TOLERANCE);

    assert!(aGauss.value(1.0, &mut aValue));
    near(aValue, (-0.5_f64).exp(), THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__VectorLambda_Value() {
    let mut aFunc = make_vector(2, |x: &Vector<'_>, y: &mut f64| {
        *y = *x.at(0).unwrap() * *x.at(0).unwrap() + *x.at(1).unwrap() * *x.at(1).unwrap();
        true
    });

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 3.0;
    aX[2] = 4.0;

    let mut aValue: f64 = 0.0;
    assert!(aFunc.value(&aX, &mut aValue));
    near(aValue, 25.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__VectorLambdaWithGradient_Values() {
    let mut aFunc = make_vector_with_gradient(
        2,
        |x: &Vector<'_>, y: &mut f64| {
            *y = *x.at(0).unwrap() * *x.at(0).unwrap() + *x.at(1).unwrap() * *x.at(1).unwrap();
            true
        },
        |x: &Vector<'_>, g: &mut Vector<'_>| {
            *g.change_at(0).unwrap() = 2.0 * *x.at(0).unwrap();
            *g.change_at(1).unwrap() = 2.0 * *x.at(1).unwrap();
            true
        },
    );

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 3.0;
    aX[2] = 4.0;

    let mut aValue: f64 = 0.0;
    let mut aGrad = Vector::with_bounds(1, 2).unwrap();
    assert!(aFunc.values(&aX, &mut aValue, &mut aGrad));
    near(aValue, 25.0, THE_TOLERANCE);
    near(aGrad[1], 6.0, THE_TOLERANCE);
    near(aGrad[2], 8.0, THE_TOLERANCE);
}
