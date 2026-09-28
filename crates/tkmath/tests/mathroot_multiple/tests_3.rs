#[test]
fn MathRoot_Multiple__FindAllRootsWithDerivative_ThreeRoots() {
    let mut aFunc = Polynomial::new(&[-6.0, 11.0, -6.0, 1.0]);

    let mut aResult =
        find_all_roots_with_derivative(&mut aFunc, 0.0, 4.0, &MultipleConfig::default());
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 3);
    near(aResult.roots[0], 1.0, THE_TOLERANCE);
    near(aResult.roots[1], 2.0, THE_TOLERANCE);
    near(aResult.roots[2], 3.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_Multiple__FindAllRootsWithDerivative_DoubleRoot() {
    let mut aFunc = Polynomial::new(&[4.0, -4.0, 1.0]);

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 100;
    aConfig.f_tolerance = 1e-8;

    let mut aResult = find_all_roots_with_derivative(&mut aFunc, 0.0, 4.0, &aConfig);
    assert!(aResult.is_done());

    assert!(aResult.nb_roots() >= 1);
    if aResult.nb_roots() > 0 {
        near(aResult.roots[0], 2.0, 1e-4);
    }
}

#[test]
fn MathRoot_Multiple__FindAllRootsWithDerivative_SineFunction() {
    let mut aSine = SineWithDerivative;

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 200;

    let mut aResult = find_all_roots_with_derivative(&mut aSine, 0.0, 7.0, &aConfig);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 3);

    near(aResult.roots[0], 0.0, THE_TOLERANCE);
    near(aResult.roots[1], std::f64::consts::PI, THE_TOLERANCE);
    near(aResult.roots[2], 2.0 * std::f64::consts::PI, THE_TOLERANCE);
}
