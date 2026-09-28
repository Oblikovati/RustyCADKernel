#[test]
fn MathInteg_ComparisonTest__Order5_Comparison() {
    let mut anOldFunc: SinFuncOld = SinFuncOld;
    let mut aNewFunc: SinFuncNew = SinFuncNew;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, std::f64::consts::PI, 5);
    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, std::f64::consts::PI, 5_usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathInteg_ComparisonTest__Order7_Comparison() {
    let mut anOldFunc: SinFuncOld = SinFuncOld;
    let mut aNewFunc: SinFuncNew = SinFuncNew;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, std::f64::consts::PI, 7);
    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, std::f64::consts::PI, 7_usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
}

#[test]
fn MathInteg_ComparisonTest__Order15_Comparison() {
    let mut anOldFunc: SinFuncOld = SinFuncOld;
    let mut aNewFunc: SinFuncNew = SinFuncNew;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, std::f64::consts::PI, 15);
    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, std::f64::consts::PI, 15_usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
}
