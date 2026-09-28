#[test]
fn MathRoot_Multiple__FindAllRoots_ThreeRoots() {
    let mut aFunc = Polynomial::new(&[-6.0, 11.0, -6.0, 1.0]);

    let mut aResult = find_all_roots(&mut aFunc, 0.0, 4.0, &MultipleConfig::default());
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 3);
    near(aResult.roots[0], 1.0, THE_TOLERANCE);
    near(aResult.roots[1], 2.0, THE_TOLERANCE);
    near(aResult.roots[2], 3.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_Multiple__FindAllRoots_SineFunction() {
    let mut aSine = Sine::default();

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 200;
    aConfig.f_tolerance = 1e-10;

    let mut aResult = find_all_roots(&mut aSine, 0.0, 10.0, &aConfig);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 4);

    near(aResult.roots[0], 0.0, THE_TOLERANCE);
    near(aResult.roots[1], std::f64::consts::PI, THE_TOLERANCE);
    near(aResult.roots[2], 2.0 * std::f64::consts::PI, THE_TOLERANCE);
    near(aResult.roots[3], 3.0 * std::f64::consts::PI, THE_TOLERANCE);
}

#[test]
fn MathRoot_Multiple__FindAllRoots_WithOffset() {
    let mut aFunc = Polynomial::new(&[0.0, 0.0, 1.0]);

    let mut aConfig = MultipleConfig::default();
    aConfig.offset = 4.0;

    let mut aResult = find_all_roots(&mut aFunc, -5.0, 5.0, &aConfig);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 2);
    near(aResult.roots[0], -2.0, THE_TOLERANCE);
    near(aResult.roots[1], 2.0, THE_TOLERANCE);
}
