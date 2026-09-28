#[test]
fn MathRoot_TrigTest__LinearSin_NoSolution() {
    let aResult: TrigResult = trigonometric_linear(1.0, -2.0, 0.0, TAU);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 0);
}

#[test]
fn MathRoot_TrigTest__LinearSin_BoundaryValue() {
    let aResult: TrigResult = trigonometric_linear(1.0, -1.0, 0.0, TAU);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], PI / 2.0, THE_TOL);
    verify_roots(&aResult, [0.0, 0.0, 0.0, 1.0, -1.0], 1e-10);
}

#[test]
fn MathRoot_TrigTest__LinearCos_ZeroConstant() {
    let aResult: TrigResult = trigonometric([0.0, 0.0, 1.0, 0.0, 0.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    verify_roots(&aResult, [0.0, 0.0, 1.0, 0.0, 0.0], 1e-10);
}
