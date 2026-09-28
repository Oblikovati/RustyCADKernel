#[test]
fn MathRoot_TrigTest__LinearSin_ZeroConstant() {
    let aResult: TrigResult = trigonometric_linear(1.0, 0.0, 0.0, TAU);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    verify_roots(&aResult, [0.0, 0.0, 0.0, 1.0, 0.0], 1e-10);
}

#[test]
fn MathRoot_TrigTest__LinearSin_HalfValue() {
    let aResult: TrigResult = trigonometric_linear(1.0, -0.5, 0.0, TAU);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    verify_roots(&aResult, [0.0, 0.0, 0.0, 1.0, -0.5], 1e-10);
}

#[test]
fn MathRoot_TrigTest__LinearSin_NegativeHalf() {
    let aResult: TrigResult = trigonometric_linear(1.0, 0.5, 0.0, TAU);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    verify_roots(&aResult, [0.0, 0.0, 0.0, 1.0, 0.5], 1e-10);
}
