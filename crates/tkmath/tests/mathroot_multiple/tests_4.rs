#[test]
fn MathRoot_Multiple__CompareWithReference_Polynomial() {
    let mut aFunc = Polynomial::new(&[-6.0, 11.0, -6.0, 1.0]);

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 50;
    aConfig.x_tolerance = 1e-10;
    aConfig.f_tolerance = 1e-10;

    let mut aResult = find_all_roots(&mut aFunc, 0.5, 3.5, &aConfig);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 3);

    for i in 0..aResult.nb_roots() {
        let mut aValue: f64 = 0.0;
        aFunc.value(aResult.roots[i], &mut aValue);
        near(aValue, 0.0, 1e-9);
    }
}

#[test]
fn MathRoot_Multiple__CompareWithReference_HigherDegree() {
    let mut aFunc = Polynomial::new(&[9.0, 0.0, -10.0, 0.0, 1.0]);

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 100;

    let mut aResult = find_all_roots(&mut aFunc, -5.0, 5.0, &aConfig);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 4);

    near(aResult.roots[0], -3.0, THE_TOLERANCE);
    near(aResult.roots[1], -1.0, THE_TOLERANCE);
    near(aResult.roots[2], 1.0, THE_TOLERANCE);
    near(aResult.roots[3], 3.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_Multiple__CompareWithReference_ClusteredTangentialRootsSparseSampling() {
    let mut aReferenceFunc = Clustered;
    let mut aCallableFunc = Clustered;

    let aReferenceRoots = FunctionRoots::with_options(
        &mut aReferenceFunc,
        0.99,
        1.02,
        4,
        [1.0e-8, 1.0e-12, 1.0e-12],
        0.0,
    );
    assert!(aReferenceRoots.is_done());
    assert!(!(aReferenceRoots.is_all_null().unwrap()));
    assert_eq!(aReferenceRoots.number_of_solutions().unwrap(), 2);

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 4;
    aConfig.x_tolerance = 1.0e-8;
    aConfig.f_tolerance = 1.0e-12;
    aConfig.null_tolerance = 1.0e-12;

    let mut aResult = find_all_roots_with_derivative(&mut aCallableFunc, 0.99, 1.02, &aConfig);
    assert!(aResult.is_done());
    assert!(!(aResult.is_all_null));
    assert_eq!(aResult.nb_roots(), 2);

    near(aResult.roots[0], aReferenceRoots.value(1).unwrap(), 1.0e-8);
    near(aResult.roots[1], aReferenceRoots.value(2).unwrap(), 1.0e-8);
}
