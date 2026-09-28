#[test]
fn MathInteg_ComparisonTest__Gauss_Cubic() {
    let mut anOldFunc: CubicFuncOld = CubicFuncOld;
    let mut aNewFunc: CubicFuncNew = CubicFuncNew;

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
    near(aNewResult.value.unwrap(), 0.25, THE_TOLERANCE);
}

#[test]
fn MathInteg_ComparisonTest__Gauss_Constant() {
    let mut anOldFunc: ConstantFuncOld = ConstantFuncOld;
    let mut aNewFunc: ConstantFuncNew = ConstantFuncNew;

    let anOrder: i32 = 3;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, 5.0, anOrder);

    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, 5.0, anOrder as usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), 5.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_ComparisonTest__Order3_Comparison() {
    let mut anOldFunc: SinFuncOld = SinFuncOld;
    let mut aNewFunc: SinFuncNew = SinFuncNew;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, std::f64::consts::PI, 3);
    let aNewResult: IntegResult = gauss(&mut aNewFunc, 0.0, std::f64::consts::PI, 3_usize);

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
}
