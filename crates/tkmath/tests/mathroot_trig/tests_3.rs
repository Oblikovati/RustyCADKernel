#[test]
fn MathRoot_TrigTest__CDE_CosPlusSinEqualsSqrt2() {
    let aResult: TrigResult = trigonometric_cde(1.0, 1.0, -(2.0_f64).sqrt(), 0.0, TAU);
    assert!(aResult.is_done());
    assert!(aResult.nb_roots >= 1);

    verify_roots(&aResult, [0.0, 0.0, 1.0, 1.0, -(2.0_f64).sqrt()], 1.0e-9);
}

#[test]
fn MathRoot_TrigTest__CDE_CosMinusSin() {
    let aResult: TrigResult = trigonometric_cde(1.0, -1.0, 0.0, 0.0, TAU);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    verify_roots(&aResult, [0.0, 0.0, 1.0, -1.0, 0.0], 1e-10);
}

#[test]
fn MathRoot_TrigTest__CDE_Harmonic() {
    let aResult: TrigResult = trigonometric_cde(3.0, 4.0, -5.0, 0.0, TAU);
    assert!(aResult.is_done());
    assert!(aResult.nb_roots >= 1);
    verify_roots(&aResult, [0.0, 0.0, 3.0, 4.0, -5.0], 1e-10);
}
