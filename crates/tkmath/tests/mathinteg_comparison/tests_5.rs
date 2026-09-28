#[test]
fn MathInteg_ComparisonTest__NegativeBounds() {
    let mut anOldFunc: QuadraticFuncOld = QuadraticFuncOld;
    let mut aNewFunc: QuadraticFuncNew = QuadraticFuncNew;

    let anOrder: i32 = 7;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, -1.0, 1.0, anOrder);

    let aNewResult: IntegResult = gauss(&mut aNewFunc, -1.0, 1.0, anOrder as usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), 2.0 / 3.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_ComparisonTest__ReversedBounds() {
    let mut anOldFunc: SinFuncOld = SinFuncOld;
    let mut aNewFunc: SinFuncNew = SinFuncNew;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, std::f64::consts::PI, 0.0, 15);
    let aNewResult: IntegResult = gauss(&mut aNewFunc, std::f64::consts::PI, 0.0, 15_usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());
    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), -2.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_ComparisonTest__PolynomialExactness_Order3() {
    let mut aFunc: Poly5Func = Poly5Func;

    let aResult: IntegResult = gauss(&mut aFunc, 0.0, 1.0, 3_usize);

    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 1.0 / 6.0, THE_TOLERANCE);
}
