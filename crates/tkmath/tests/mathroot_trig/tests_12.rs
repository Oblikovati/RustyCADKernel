#[test]
fn MathRoot_TrigTest__GlobalScalingPreservesClassificationAndRoots() {
    let aSmall: TrigResult = trigonometric([1.0e-13, 0.0, 0.0, 0.0, 1.0e-13], 0.0, TAU, 1.5e-12);
    let aUnit: TrigResult = trigonometric([1.0, 0.0, 0.0, 0.0, 1.0], 0.0, TAU, 1.5e-12);
    assert!(aSmall.is_done());
    assert!(aUnit.is_done());
    assert!(!(aSmall.infinite_roots));
    assert!(!(aUnit.infinite_roots));
    assert_eq!(aSmall.nb_roots, 0);
    assert_eq!(aSmall.nb_roots, aUnit.nb_roots);

    let aScaledLinear: TrigResult = trigonometric_linear(1.0e-200, -0.5e-200, 0.0, TAU);
    let aLinear: TrigResult = trigonometric_linear(1.0, -0.5, 0.0, TAU);
    assert!(aScaledLinear.is_done());
    assert_eq!(aScaledLinear.nb_roots, aLinear.nb_roots);
    for anIndex in 0..aLinear.nb_roots {
        near(
            aScaledLinear.roots[anIndex],
            aLinear.roots[anIndex],
            THE_TOL,
        );
    }
}

#[test]
fn MathRoot_TrigTest__Stability_MixedMagnitude() {
    let aResult: TrigResult = trigonometric([1.0, 0.0, 1.0e6, 0.0, -0.5e6], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());
    verify_roots(&aResult, [1.0, 0.0, 1.0e6, 0.0, -0.5e6], 1.0e-4);
}

#[test]
fn MathRoot_TrigTest__RootOrdering_Sorted() {
    let aResult: TrigResult = trigonometric_linear(1.0, 0.0, 0.0, TAU);
    assert!(aResult.is_done());
    for i in 1..aResult.nb_roots {
        assert!(aResult.roots[i - 1] <= aResult.roots[i]);
    }
}
