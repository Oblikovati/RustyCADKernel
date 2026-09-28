#[test]
fn MathRoot_TrigTest__Bounds_MultipleCircles() {
    let aResult: TrigResult = trigonometric_linear(1.0, 0.0, 0.0, 4.0 * PI);
    assert!(aResult.is_done());

    assert_eq!(aResult.nb_roots, 2);
}

#[test]
fn MathRoot_TrigTest__CosineOnlyReturnsOnePeriod() {
    let aResult: TrigResult = trigonometric([0.0, 0.0, 1.0, 0.0, 0.0], 0.0, 4.0 * PI, 1.5e-12);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
}

#[test]
fn MathRoot_TrigTest__Degenerate_AllZero() {
    let aResult: TrigResult = trigonometric([0.0, 0.0, 0.0, 0.0, 0.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.infinite_roots);
}
