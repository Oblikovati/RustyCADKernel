#[test]
fn MathRoot_Multiple__InvalidBoundsAndSampleCountsAreRejectedBeforeSampling() {
    let mut aFunc = Polynomial::new(&[-1.0, 1.0]);
    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 1;
    assert_eq!(
        find_all_roots(&mut aFunc, 0.0, 2.0, &aConfig).status,
        Status::InvalidInput
    );

    aConfig.nb_samples = 20;
    assert_eq!(
        find_all_roots(&mut aFunc, 2.0, 0.0, &aConfig).status,
        Status::InvalidInput
    );
    assert_eq!(
        find_all_roots(&mut aFunc, 0.0, f64::INFINITY, &aConfig).status,
        Status::InvalidInput
    );

    let anAllResult = find_all_roots_with_intervals(&mut aFunc, 0.0, 2.0, 1, [1e-10; 3]);
    assert_eq!(anAllResult.status, Status::InvalidInput);
}

#[test]
fn MathRoot_Multiple__ValueOnlySearchDoesNotClaimUnsampledTangentialRoot() {
    let aRoot: f64 = (2.0_f64).sqrt() / 4.0;
    let mut aFunc = make_scalar(|theX: f64, theY: &mut f64| {
        let aDx: f64 = theX - aRoot;
        *theY = aDx * aDx;
        true
    });

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 2;
    aConfig.f_tolerance = 1.0e-14;
    let aResult = find_all_roots(&mut aFunc, 0.0, 1.0, &aConfig);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 0);
}

#[test]
fn MathRoot_Multiple__FailedRefinementIsPropagatedAndEarlierRootsArePreserved() {
    let mut aFunc = FailOnSecondBracket::default();

    let mut aConfig = MultipleConfig::default();
    aConfig.nb_samples = 2;
    let aResult = find_all_roots(&mut aFunc, -1.0, 1.0, &aConfig);
    assert_eq!(aResult.status, Status::CallbackError);
    assert_eq!(aResult.nb_roots(), 1);
    near(aResult.roots[0], -0.5, THE_TOLERANCE);
}
