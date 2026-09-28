#[test]
fn MathRoot_TrigTest__Result_BoolConversion() {
    let aResult: TrigResult = trigonometric_linear(1.0, 0.0, 0.0, TAU);
    assert!(bool::from(&aResult));
    assert!(aResult.is_done());
}

#[test]
fn MathRoot_TrigTest__Result_InfiniteRoots() {
    let aResult: TrigResult = trigonometric([0.0, 0.0, 0.0, 0.0, 0.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.infinite_roots);
}

#[test]
fn MathRoot_TrigTest__Ellipse_PointOnMajorAxis() {
    let aB2MinusA2Over2: f64 = (100.0 - 400.0) / 2.0;
    let aAX: f64 = 20.0 * 30.0;
    let aBY: f64 = 0.0;

    let aResult: TrigResult =
        trigonometric([0.0, aB2MinusA2Over2, -aBY, aAX, 0.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());

    assert!(aResult.nb_roots >= 2);

    verify_roots(&aResult, [0.0, aB2MinusA2Over2, -aBY, aAX, 0.0], 1e-10);
}
