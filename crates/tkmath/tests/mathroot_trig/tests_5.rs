#[test]
fn MathRoot_TrigTest__QuadraticCos_TwoSolutions() {
    let aResult: TrigResult = trigonometric([1.0, 0.0, 0.0, 0.0, -1.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());
    assert!(aResult.nb_roots >= 2);
    verify_roots(&aResult, [1.0, 0.0, 0.0, 0.0, -1.0], 1e-10);
}

#[test]
fn MathRoot_TrigTest__QuadraticCos_FourSolutions() {
    let aResult: TrigResult = trigonometric([1.0, 0.0, 0.0, 0.0, -0.25], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 4);
    verify_roots(&aResult, [1.0, 0.0, 0.0, 0.0, -0.25], 1e-10);
}

#[test]
fn MathRoot_TrigTest__CosSinProduct_AE_Zero() {
    let aResult: TrigResult = trigonometric([0.0, 1.0, 0.0, 0.0, 0.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 4);
    verify_roots(&aResult, [0.0, 1.0, 0.0, 0.0, 0.0], 1e-10);
}
