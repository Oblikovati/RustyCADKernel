#[test]
fn MathRoot_TrigTest__CosineRootAtLowerBound() {
    let aLower: f64 = PI / 3.0;
    let aResult: TrigResult = trigonometric([0.0, 0.0, 1.0, 0.0, -0.5], aLower, PI, 1.5e-12);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], aLower, THE_TOL);
}

#[test]
fn MathRoot_TrigTest__Bounds_RootOutsideRange() {
    let aResult: TrigResult = trigonometric_linear(1.0, -0.5, PI, TAU);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 0);
}

#[test]
fn MathRoot_TrigTest__Bounds_FullCircle() {
    let aResult: TrigResult = trigonometric_linear(1.0, 0.0, 0.0, TAU);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
}
