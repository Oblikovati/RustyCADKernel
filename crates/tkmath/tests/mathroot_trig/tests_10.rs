#[test]
fn MathRoot_TrigTest__Degenerate_ConstantNonZero() {
    let aResult: TrigResult = trigonometric([0.0, 0.0, 0.0, 0.0, 1.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 0);
}

#[test]
fn MathRoot_TrigTest__Degenerate_NearZeroCoefficients() {
    let aResult: TrigResult = trigonometric([1.0e-14, 1.0e-14, 1.0, 0.0, -0.5], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());

    assert_eq!(aResult.nb_roots, 2);
    verify_roots(&aResult, [1.0e-14, 1.0e-14, 1.0, 0.0, -0.5], 1.0e-6);
}

#[test]
fn MathRoot_TrigTest__SpecialCase_PiRoot() {
    let aResult: TrigResult = trigonometric([1.0, 0.0, -1.0, 0.0, 0.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());
    assert!(aResult.nb_roots >= 2);
    verify_roots(&aResult, [1.0, 0.0, -1.0, 0.0, 0.0], 1e-10);
}
