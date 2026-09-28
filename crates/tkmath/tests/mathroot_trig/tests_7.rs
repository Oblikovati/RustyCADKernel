#[test]
fn MathRoot_TrigTest__Bounds_FirstQuadrant() {
    let aResult: TrigResult = trigonometric_linear(1.0, -0.5, 0.0, PI / 2.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    assert!(aResult.roots[0] >= 0.0 - THE_TOL);
    assert!(aResult.roots[0] <= PI / 2.0 + THE_TOL);
    verify_roots(&aResult, [0.0, 0.0, 0.0, 1.0, -0.5], 1e-10);
}

#[test]
fn MathRoot_TrigTest__Bounds_SecondQuadrant() {
    let aResult: TrigResult = trigonometric_linear(1.0, -0.5, PI / 2.0, PI);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    assert!(aResult.roots[0] >= PI / 2.0 - THE_TOL);
    assert!(aResult.roots[0] <= PI + THE_TOL);
    verify_roots(&aResult, [0.0, 0.0, 0.0, 1.0, -0.5], 1e-10);
}

#[test]
fn MathRoot_TrigTest__Bounds_NarrowRange() {
    let aResult: TrigResult = trigonometric(
        [0.0, 0.0, 1.0, 0.0, 0.0],
        PI / 2.0 - 0.1,
        PI / 2.0 + 0.1,
        1.5e-12,
    );
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], PI / 2.0, THE_TOL);
}
