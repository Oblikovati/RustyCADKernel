#[test]
fn MathUtils_Functor_Vector__Rosenbrock_Value() {
    let mut aRosen = Rosenbrock::default();

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 1.0;
    aX[2] = 1.0;

    let mut aValue: f64 = 0.0;
    assert!(aRosen.value(&aX, &mut aValue));
    near(aValue, 0.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__Rosenbrock_AwayFromMinimum() {
    let mut aRosen = Rosenbrock::default();

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 0.0;
    aX[2] = 0.0;

    let mut aValue: f64 = 0.0;
    assert!(aRosen.value(&aX, &mut aValue));
    near(aValue, 1.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__Rosenbrock_Gradient() {
    let mut aRosen = Rosenbrock::default();

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 1.0;
    aX[2] = 1.0;

    let mut aGrad = Vector::with_bounds(1, 2).unwrap();
    assert!(aRosen.gradient(&aX, &mut aGrad));
    near(aGrad[1], 0.0, THE_TOLERANCE);
    near(aGrad[2], 0.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__Sphere_Value() {
    let mut aSphere = Sphere::new(3);

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aX[1] = 1.0;
    aX[2] = 2.0;
    aX[3] = 3.0;

    let mut aValue: f64 = 0.0;
    assert!(aSphere.value(&aX, &mut aValue));
    near(aValue, 14.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__Sphere_Gradient() {
    let mut aSphere = Sphere::new(3);

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aX[1] = 1.0;
    aX[2] = 2.0;
    aX[3] = 3.0;

    let mut aGrad = Vector::with_bounds(1, 3).unwrap();
    assert!(aSphere.gradient(&aX, &mut aGrad));
    near(aGrad[1], 2.0, THE_TOLERANCE);
    near(aGrad[2], 4.0, THE_TOLERANCE);
    near(aGrad[3], 6.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__Booth_Value() {
    let mut aBooth = Booth;

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 1.0;
    aX[2] = 3.0;

    let mut aValue: f64 = 0.0;
    assert!(aBooth.value(&aX, &mut aValue));
    near(aValue, 0.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Vector__Beale_Value() {
    let mut aBeale = Beale;

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 3.0;
    aX[2] = 0.5;

    let mut aValue: f64 = 0.0;
    assert!(aBeale.value(&aX, &mut aValue));
    near(aValue, 0.0, THE_TOLERANCE);
}
