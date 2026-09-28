#[test]
fn MathUtils_Functor_Scalar__Shifted_Value() {
    let mut aF = Polynomial::new(&[0.0, 0.0, 1.0]);
    let mut aShifted = make_shifted(aF, 5.0);

    let mut aValue: f64 = 0.0;
    assert!(aShifted.value(2.0, &mut aValue));
    near(aValue, 9.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Negated_Value() {
    let mut aF = Polynomial::new(&[0.0, 0.0, 1.0]);
    let mut aNegated = make_negated(aF);

    let mut aValue: f64 = 0.0;
    assert!(aNegated.value(3.0, &mut aValue));
    near(aValue, -9.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Constant_Value() {
    let mut aConst = Constant::new(42.0);

    let mut aValue: f64 = 0.0;
    let mut aDeriv: f64 = 0.0;
    assert!(aConst.value(123.456, &mut aValue));
    near(aValue, 42.0, THE_TOLERANCE);

    assert!(aConst.values(789.0, &mut aValue, &mut aDeriv));
    near(aValue, 42.0, THE_TOLERANCE);
    near(aDeriv, 0.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Linear_Value() {
    let mut aLinear = Linear::new(2.0, 3.0);

    let mut aValue: f64 = 0.0;
    let mut aDeriv: f64 = 0.0;
    assert!(aLinear.value(5.0, &mut aValue));
    near(aValue, 13.0, THE_TOLERANCE);

    assert!(aLinear.values(5.0, &mut aValue, &mut aDeriv));
    near(aValue, 13.0, THE_TOLERANCE);
    near(aDeriv, 2.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Sine_Value() {
    let mut aSine = Sine::new(1.0, 1.0, 0.0, 0.0);

    let mut aValue: f64 = 0.0;
    assert!(aSine.value(0.0, &mut aValue));
    near(aValue, 0.0, THE_TOLERANCE);

    assert!(aSine.value(std::f64::consts::PI / 2.0, &mut aValue));
    near(aValue, 1.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Sine_Values() {
    let mut aSine = Sine::new(2.0, 1.0, 0.0, 0.0);

    let mut aValue: f64 = 0.0;
    let mut aDeriv: f64 = 0.0;
    assert!(aSine.values(0.0, &mut aValue, &mut aDeriv));
    near(aValue, 0.0, THE_TOLERANCE);
    near(aDeriv, 2.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Scalar__Cosine_Value() {
    let mut aCosine = Cosine::new(1.0, 1.0, 0.0, 0.0);

    let mut aValue: f64 = 0.0;
    assert!(aCosine.value(0.0, &mut aValue));
    near(aValue, 1.0, THE_TOLERANCE);

    assert!(aCosine.value(std::f64::consts::PI, &mut aValue));
    near(aValue, -1.0, THE_TOLERANCE);
}
