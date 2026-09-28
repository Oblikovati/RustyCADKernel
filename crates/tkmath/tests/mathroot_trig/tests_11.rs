#[test]
fn MathRoot_TrigTest__SpecialCase_PiRootCheck() {
    let aResult: TrigResult = trigonometric([1.0, 0.0, 0.0, 0.0, -1.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());

    let mut aFoundPi: bool = false;
    for i in 0..aResult.nb_roots {
        if (aResult.roots[i] - PI).abs() < THE_TOL {
            aFoundPi = true;
            break;
        }
    }
    assert!(aFoundPi);
    verify_roots(&aResult, [1.0, 0.0, 0.0, 0.0, -1.0], 1e-10);
}

#[test]
fn MathRoot_TrigTest__Stability_LargeCoefficients() {
    let aResult: TrigResult =
        trigonometric([1000.0, 500.0, 300.0, 400.0, -200.0], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());
    verify_roots(&aResult, [1000.0, 500.0, 300.0, 400.0, -200.0], 1.0e-6);
}

#[test]
fn MathRoot_TrigTest__Stability_SmallCoefficients() {
    let aResult: TrigResult =
        trigonometric([1.0e-6, 0.5e-6, 0.3e-6, 0.4e-6, -0.2e-6], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());

    if aResult.nb_roots > 0 {
        verify_roots(&aResult, [1.0e-6, 0.5e-6, 0.3e-6, 0.4e-6, -0.2e-6], 1.0e-4);
    }
}
