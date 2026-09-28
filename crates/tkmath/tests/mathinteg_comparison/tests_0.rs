#[test]
fn MathInteg_ComparisonTest__Gauss_SinPi() {
    let mut anOldFunc: SinFuncOld = SinFuncOld;
    let mut aNewFunc: SinFuncNew = SinFuncNew;

    let anOrder: i32 = 15;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, std::f64::consts::PI, anOrder);

    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, std::f64::consts::PI, anOrder as usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), 2.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_ComparisonTest__Gauss_Quadratic() {
    let mut anOldFunc: QuadraticFuncOld = QuadraticFuncOld;
    let mut aNewFunc: QuadraticFuncNew = QuadraticFuncNew;

    let anOrder: i32 = 7;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, 1.0, anOrder);

    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, 1.0, anOrder as usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), 1.0 / 3.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_ComparisonTest__Gauss_Exponential() {
    let mut anOldFunc: ExpFuncOld = ExpFuncOld;
    let mut aNewFunc: ExpFuncNew = ExpFuncNew;

    let anOrder: i32 = 15;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, 1.0, anOrder);

    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, 1.0, anOrder as usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(
        aNewResult.value.unwrap(),
        (1.0_f64).exp() - 1.0,
        THE_TOLERANCE,
    );
}
