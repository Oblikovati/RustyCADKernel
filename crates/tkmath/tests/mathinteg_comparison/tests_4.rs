#[test]
fn MathInteg_ComparisonTest__Order21_Comparison() {
    let mut anOldFunc: SinFuncOld = SinFuncOld;
    let mut aNewFunc: SinFuncNew = SinFuncNew;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, std::f64::consts::PI, 21);
    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, std::f64::consts::PI, 21_usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathInteg_ComparisonTest__Order41_Comparison() {
    let mut anOldFunc: SinFuncOld = SinFuncOld;
    let mut aNewFunc: SinFuncNew = SinFuncNew;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, std::f64::consts::PI, 41);
    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, std::f64::consts::PI, 41_usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());
    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathInteg_ComparisonTest__HigherOrderMoreAccurate() {
    let mut aFunc: ExpFuncNew = ExpFuncNew;

    let aExact: f64 = (1.0_f64).exp() - 1.0;

    let aResult3: IntegResult = gauss(&mut aFunc, 0.0, 1.0, 3_usize);
    let aResult7: IntegResult = gauss(&mut aFunc, 0.0, 1.0, 7_usize);
    let aResult15: IntegResult = gauss(&mut aFunc, 0.0, 1.0, 15_usize);

    assert!(aResult3.is_done());
    assert!(aResult7.is_done());
    assert!(aResult15.is_done());

    let aError3: f64 = (aResult3.value.unwrap() - aExact).abs();
    let aError7: f64 = (aResult7.value.unwrap() - aExact).abs();
    let aError15: f64 = (aResult15.value.unwrap() - aExact).abs();

    assert!(aError7 <= aError3 + 1.0e-15);
    assert!(aError15 <= aError7 + 1.0e-15);
}
