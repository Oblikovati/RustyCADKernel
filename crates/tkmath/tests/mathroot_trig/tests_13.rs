#[test]
fn MathRoot_TrigTest__RootUniqueness_NoDuplicates() {
    let aResult: TrigResult = trigonometric([1.0, 0.0, 0.0, 0.0, -0.25], 0.0, TAU, 1.5e-12);
    assert!(aResult.is_done());
    for i in 0..aResult.nb_roots {
        for j in i + 1..aResult.nb_roots {
            assert!((aResult.roots[i] - aResult.roots[j]).abs() > 1.0e-10);
        }
    }
}

#[test]
fn MathRoot_TrigTest__Wrapper_TrigonometricLinear() {
    let aResult1: TrigResult = trigonometric_linear(2.0, -1.0, 0.0, TAU);
    let aResult2: TrigResult = trigonometric([0.0, 0.0, 0.0, 2.0, -1.0], 0.0, TAU, 1.5e-12);

    assert_eq!(aResult1.nb_roots, aResult2.nb_roots);
    for i in 0..aResult1.nb_roots {
        near(aResult1.roots[i], aResult2.roots[i], THE_TOL);
    }
}

#[test]
fn MathRoot_TrigTest__Wrapper_TrigonometricCDE() {
    let aResult1: TrigResult = trigonometric_cde(1.0, 2.0, -0.5, 0.0, TAU);
    let aResult2: TrigResult = trigonometric([0.0, 0.0, 1.0, 2.0, -0.5], 0.0, TAU, 1.5e-12);

    assert_eq!(aResult1.nb_roots, aResult2.nb_roots);
    for i in 0..aResult1.nb_roots {
        near(aResult1.roots[i], aResult2.roots[i], THE_TOL);
    }
}
