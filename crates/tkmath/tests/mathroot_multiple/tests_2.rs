#[test]
fn MathRoot_Multiple__FindAllRoots_CloseRoots() {
    let mut aFunc = Polynomial::new(&[1.01, -2.01, 1.0]);

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 500;
    aConfig.x_tolerance = 1e-4;

    let mut aResult = find_all_roots(&mut aFunc, 0.0, 2.0, &aConfig);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 2);
    near(aResult.roots[0], 1.0, 1e-3);
    near(aResult.roots[1], 1.01, 1e-3);
}

#[test]
fn MathRoot_Multiple__FindAllRoots_NullFunction() {
    let mut aFunc = Constant::new(0.0);

    let mut aConfig = MultipleConfig::default();
    aConfig.null_tolerance = 1e-10;

    let mut aResult = find_all_roots(&mut aFunc, -1.0, 1.0, &aConfig);
    assert!(aResult.is_done());
    assert!(aResult.is_all_null);
}

#[test]
fn MathRoot_Multiple__FindAllRoots_LambdaFunction() {
    let mut aFunc = make_scalar(|x: f64, y: &mut f64| {
        *y = (x).cos() - 0.5;
        true
    });

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 100;

    let mut aResult = find_all_roots(&mut aFunc, 0.0, 2.0 * std::f64::consts::PI, &aConfig);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 2);

    near(aResult.roots[0], std::f64::consts::PI / 3.0, THE_TOLERANCE);
    near(
        aResult.roots[1],
        5.0 * std::f64::consts::PI / 3.0,
        THE_TOLERANCE,
    );
}
