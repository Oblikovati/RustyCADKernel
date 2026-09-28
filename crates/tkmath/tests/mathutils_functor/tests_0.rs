#[test]
fn MathUtils_Functor_Scalar__ScalarLambda_Value() {
    let mut aFunc = make_scalar(|x: f64, y: &mut f64| {
        *y = x * x - 2.0;
        true
    });

    let mut aValue: f64 = 0.0;
    assert!(aFunc.value(2.0, &mut aValue));
    near(aValue, 2.0, THE_TOLERANCE);

    assert!(aFunc.value((2.0_f64).sqrt(), &mut aValue));
    near(aValue, 0.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__ScalarLambdaWithDerivative_Values() {
    let mut aFunc = make_scalar_with_derivative(|x: f64, y: &mut f64, dy: &mut f64| {
        *y = x * x - 2.0;
        *dy = 2.0 * x;
        true
    });

    let mut aValue: f64 = 0.0;
    let mut aDeriv: f64 = 0.0;
    assert!(aFunc.values(3.0, &mut aValue, &mut aDeriv));
    near(aValue, 7.0, THE_TOLERANCE);
    near(aDeriv, 6.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Polynomial_Value() {
    let mut aPoly = Polynomial::new(&[1.0, 2.0, 3.0]);

    let mut aValue: f64 = 0.0;
    assert!(aPoly.value(0.0, &mut aValue));
    near(aValue, 1.0, THE_TOLERANCE);

    assert!(aPoly.value(1.0, &mut aValue));
    near(aValue, 6.0, THE_TOLERANCE);

    assert!(aPoly.value(2.0, &mut aValue));
    near(aValue, 17.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Polynomial_Values() {
    let mut aPoly = Polynomial::new(&[-2.0, 0.0, 1.0]);

    let mut aValue: f64 = 0.0;
    let mut aDeriv: f64 = 0.0;
    assert!(aPoly.values(3.0, &mut aValue, &mut aDeriv));
    near(aValue, 7.0, THE_TOLERANCE);
    near(aDeriv, 6.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Polynomial_WithRoots() {
    let mut aPoly = Polynomial::new(&[-2.0, 0.0, 1.0]);

    let mut aResult = brent(&mut aPoly, 0.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    assert!(aResult.root.is_some());
    near(aResult.root.unwrap(), (2.0_f64).sqrt(), 1e-9);
}

#[test]
fn MathUtils_Functor_Scalar__Polynomial_WithNewton() {
    let mut aPoly = Polynomial::new(&[-5.0, -2.0, 0.0, 1.0]);

    let mut aResult = newton(&mut aPoly, 2.0, &Config::default());
    assert!(aResult.is_done());
    assert!(aResult.root.is_some());

    near(aResult.root.unwrap(), 2.0945514815, 1e-8);
}

#[test]
fn MathUtils_Functor_Scalar__Rational_Value() {
    let mut aRat = Rational::new(&[1.0, 1.0], &[1.0, 0.0, 1.0]);

    let mut aValue: f64 = 0.0;
    assert!(aRat.value(0.0, &mut aValue));
    near(aValue, 1.0, THE_TOLERANCE);

    assert!(aRat.value(1.0, &mut aValue));
    near(aValue, 1.0, THE_TOLERANCE);

    assert!(aRat.value(2.0, &mut aValue));
    near(aValue, 0.6, THE_TOLERANCE);
}
