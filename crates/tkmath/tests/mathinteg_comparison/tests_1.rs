#[test]
fn MathInteg_ComparisonTest__Gauss_Cosine() {
    let mut anOldFunc: CosFuncOld = CosFuncOld;
    let mut aNewFunc: CosFuncNew = CosFuncNew;

    let anOrder: i32 = 15;

    let anOldInteg: GaussSingleIntegration =
        GaussSingleIntegration::new(&mut anOldFunc, 0.0, std::f64::consts::PI / 2.0, anOrder);

    let aNewResult: IntegResult = gauss(
        &mut aNewFunc,
        0.0,
        std::f64::consts::PI / 2.0,
        anOrder as usize,
    );

    assert!(anOldInteg.is_done());
    assert!(aNewResult.is_done());

    near(
        anOldInteg.value().unwrap(),
        aNewResult.value.unwrap(),
        THE_TOLERANCE,
    );
    near(aNewResult.value.unwrap(), 1.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_ComparisonTest__Gauss_LogIntegrand() {
    let mut anOldFunc: LogIntegrandOld = LogIntegrandOld;
    let mut aNewFunc: LogIntegrandNew = LogIntegrandNew;

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
    near(aNewResult.value.unwrap(), (2.0_f64).ln(), THE_TOLERANCE);
}

#[test]
fn MathInteg_ComparisonTest__Gauss_Linear() {
    let mut anOldFunc: LinearFuncOld = LinearFuncOld;
    let mut aNewFunc: LinearFuncNew = LinearFuncNew;

    let anOrder: i32 = 3;

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
    near(aNewResult.value.unwrap(), 0.5, THE_TOLERANCE);
}
