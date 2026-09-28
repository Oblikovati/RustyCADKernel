#[test]
fn MathRoot_TrigTest__Ellipse_GeneralPoint() {
    let aB2MinusA2Over2: f64 = (100.0 - 400.0) / 2.0;
    let aAX: f64 = 20.0 * 15.0;
    let aBY: f64 = 10.0 * 8.0;

    let aResult: TrigResult =
        trigonometric([0.0, aB2MinusA2Over2, -aBY, aAX, 0.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());

    verify_roots(&aResult, [0.0, aB2MinusA2Over2, -aBY, aAX, 0.0], 1e-10);
}

#[test]
fn MathRoot_TrigTest__NegativeBounds_Basic() {
    let aResult: TrigResult = trigonometric_linear(1.0, 0.0, -PI, 0.0);
    assert!(aResult.is_done());

    assert!(aResult.nb_roots >= 1);
    for i in 0..aResult.nb_roots {
        assert!(aResult.roots[i] >= -PI - THE_TOL);
        assert!(aResult.roots[i] <= THE_TOL);
    }
}

#[test]
fn MathRoot_TrigTest__NegativeBounds_CosEquation() {
    let aResult: TrigResult = trigonometric([0.0, 0.0, 1.0, 0.0, -0.5], -PI, PI, 1.5e-12);
    assert!(aResult.is_done());

    assert!(aResult.nb_roots >= 2);
    verify_roots(&aResult, [0.0, 0.0, 1.0, 0.0, -0.5], 1e-10);
}
