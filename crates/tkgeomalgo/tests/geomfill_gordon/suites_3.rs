// GeomFill_NetworkSurface tests and rational GeomFill_Gordon tests, part 3.

// `aNetwork.Init(Profiles, Guides, ProfileParams, GuideParams, makeIntersectionGrid(..),
// Weights, Tolerance, IsUClosed, IsVClosed); aNetwork.Perform();`.
fn network_of(
    (aProfiles, aGuides): (&Array1<'static, Handle<GeomBSplineCurve>>, &Array1<'static, Handle<GeomBSplineCurve>>),
    (aProfileParams, aGuideParams): (&Array1<'static, f64>, &Array1<'static, f64>),
    aWeights: &Array2<'static, f64>,
    theTolerance: f64,
    theClosed: (bool, bool),
) -> GeomFillNetworkSurface {
    let mut aNetwork = GeomFillNetworkSurface::new();
    aNetwork.init(
        (aProfiles, aGuides),
        (aProfileParams, aGuideParams),
        (&makeIntersectionGrid(aProfiles, aGuideParams), aWeights),
        theTolerance,
        theClosed,
    );
    aNetwork.perform();
    aNetwork
}

// The same network with a unit weight grid.
fn unit_weight_network_of(
    aCurves: (&Array1<'static, Handle<GeomBSplineCurve>>, &Array1<'static, Handle<GeomBSplineCurve>>),
    aParams: (&Array1<'static, f64>, &Array1<'static, f64>),
    theTolerance: f64,
    theClosed: (bool, bool),
) -> GeomFillNetworkSurface {
    let aWeights = makeUnitWeightGrid(aCurves.0, aParams.1);
    network_of(aCurves, aParams, &aWeights, theTolerance, theClosed)
}

// The degree-9 rational line network of the rational degree overflow tests.
fn rational_line_network(aDegree: i32) -> (Vec<Handle<GeomBSplineCurve>>, Vec<Handle<GeomBSplineCurve>>) {
    let aProfiles = vec![
        makeRationalLineBSpline(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), aDegree),
        makeRationalLineBSpline(p(0.0, 0.5, 0.0), p(1.0, 0.5, 0.0), aDegree),
        makeRationalLineBSpline(p(0.0, 1.0, 0.0), p(1.0, 1.0, 0.0), aDegree),
    ];
    let aGuides = vec![
        makeRationalLineBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0), aDegree),
        makeRationalLineBSpline(p(0.5, 0.0, 0.0), p(0.5, 1.0, 0.0), aDegree),
        makeRationalLineBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0), aDegree),
    ];
    (aProfiles, aGuides)
}

// The rational quadratic line profiles and linear guides of the exact rational
// reparametrization tests.
fn rational_quadratic_line_network() -> CurveNetwork {
    let aProfiles = curves(&[
        makeRationalQuadraticLineBSpline(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), 1.0, 0.45, 1.0),
        makeRationalQuadraticLineBSpline(p(0.0, 0.4, 0.0), p(1.0, 0.4, 0.0), 1.0, 1.8, 0.7),
        makeRationalQuadraticLineBSpline(p(0.0, 1.0, 0.0), p(1.0, 1.0, 0.0), 0.8, 0.65, 1.5),
    ]);
    let aGuides = curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(0.4, 0.0, 0.0), p(0.4, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    (aProfiles, aGuides)
}

#[test]
fn NetworkSurface_UClosedNetwork_ProducesUPeriodicSurface() {
    let aProfiles = bsplines(&[
        makeQuadraticBSpline(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(0.0, 0.0, 0.0)),
        makeQuadraticBSpline(p(0.0, 1.0, 0.0), p(1.0, 1.0, 1.0), p(0.0, 1.0, 0.0)),
        makeQuadraticBSpline(p(0.0, 2.0, 0.0), p(1.0, 2.0, 0.0), p(0.0, 2.0, 0.0)),
    ]);
    let aGuides = bsplines(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 2.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 2.0, 0.0)),
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 2.0, 0.0)),
    ]);
    let aProfileParams = reals(&[0.0, 0.5, 1.0]);
    let aGuideParams = reals(&[0.0, 0.5, 1.0]);

    let aNetwork = unit_weight_network_of(
        (&aProfiles, &aGuides),
        (&aProfileParams, &aGuideParams),
        precision::confusion(),
        (true, false),
    );

    assert!(aNetwork.is_done());

    let aSurf = aNetwork.surface().unwrap();
    assert!(!aSurf.is_null());
    assert!(aSurf.is_u_periodic());
}

#[test]
fn NetworkSurface_UClosedNetwork_UsesInputToleranceForPeriodicity() {
    const aSeamGap: f64 = 1.0e-5;
    const aTolerance: f64 = 1.0e-4;

    let aProfiles = bsplines(&[
        makeQuadraticBSpline(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(aSeamGap, 0.0, 0.0)),
        makeQuadraticBSpline(p(0.0, 1.0, 0.0), p(1.0, 1.0, 1.0), p(aSeamGap, 1.0, 0.0)),
        makeQuadraticBSpline(p(0.0, 2.0, 0.0), p(1.0, 2.0, 0.0), p(aSeamGap, 2.0, 0.0)),
    ]);
    let aGuides = bsplines(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 2.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 2.0, 0.0)),
        makeLinearBSpline(p(aSeamGap, 0.0, 0.0), p(aSeamGap, 2.0, 0.0)),
    ]);
    let aProfileParams = reals(&[0.0, 0.5, 1.0]);
    let aGuideParams = reals(&[0.0, 0.5, 1.0]);

    let aNetwork =
        unit_weight_network_of((&aProfiles, &aGuides), (&aProfileParams, &aGuideParams), aTolerance, (true, false));

    assert!(aNetwork.is_done(), "{:?}", aNetwork.status());
    assert!(aNetwork.surface().unwrap().is_u_periodic());
}

#[test]
fn NetworkSurface_InvalidPreparedNetworkReportsStatus() {
    let aProfiles = bsplines(&[makeLinearBSpline(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0))]);
    let aGuides = bsplines(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    let aProfileParams = reals(&[0.0]);
    let aGuideParams = reals(&[0.0, 1.0]);

    let mut aNetwork = GeomFillNetworkSurface::new();
    aNetwork.init(
        (&aProfiles, &aGuides),
        (&aProfileParams, &aGuideParams),
        (&makeIntersectionGrid(&aProfiles, &aGuideParams), &makeUnitWeightGrid(&aProfiles, &aGuideParams)),
        precision::confusion(),
        (false, false),
    );
    assert_eq!(aNetwork.status(), NetworkSurfaceStatus::NotStarted);

    aNetwork.perform();

    assert!(!aNetwork.is_done());
    assert_eq!(aNetwork.status(), NetworkSurfaceStatus::InvalidInput);
    let error = aNetwork.surface().err().map(|failure| failure.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("StdFail_NotDone"));
}

#[test]
fn NetworkSurface_InvalidWeightsReportsInvalidInput() {
    let aProfiles = bsplines(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0)),
        makeLinearBSpline(p(0.0, 1.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    let aGuides = bsplines(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    let aProfileParams = reals(&[0.0, 1.0]);
    let aGuideParams = reals(&[0.0, 1.0]);

    let mut aWeights = makeUnitWeightGrid(&aProfiles, &aGuideParams);
    aWeights[(2, 2)] = 0.0;

    let aNetwork = network_of(
        (&aProfiles, &aGuides),
        (&aProfileParams, &aGuideParams),
        &aWeights,
        precision::confusion(),
        (false, false),
    );

    assert!(!aNetwork.is_done());
    assert_eq!(aNetwork.status(), NetworkSurfaceStatus::InvalidInput);
}

#[test]
fn NetworkSurface_CompatibleRationalWeightsProducesRationalSurface() {
    let aProfiles = bsplines(&[
        makeRationalQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.5, 0.0, 0.0), p(1.0, 0.0, 0.0), 0.5),
        makeRationalQuadraticBSpline(p(0.0, 0.5, 0.0), p(0.5, 0.5, 0.0), p(1.0, 0.5, 0.0), 0.5),
        makeRationalQuadraticBSpline(p(0.0, 1.0, 0.0), p(0.5, 1.0, 0.0), p(1.0, 1.0, 0.0), 0.5),
    ]);
    let aGuides = bsplines(&[
        makeWeightedLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0), 1.0),
        makeWeightedLinearBSpline(p(0.5, 0.0, 0.0), p(0.5, 1.0, 0.0), 0.75),
        makeWeightedLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0), 1.0),
    ]);
    let aProfileParams = reals(&[0.0, 0.5, 1.0]);
    let aGuideParams = reals(&[0.0, 0.5, 1.0]);

    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for aProfileIdx in 1..=3 {
        aWeights[(1, aProfileIdx)] = 1.0;
        aWeights[(2, aProfileIdx)] = 0.75;
        aWeights[(3, aProfileIdx)] = 1.0;
    }

    let aNetwork = network_of(
        (&aProfiles, &aGuides),
        (&aProfileParams, &aGuideParams),
        &aWeights,
        precision::confusion(),
        (false, false),
    );

    assert!(aNetwork.is_done());

    let aSurface = aNetwork.surface().unwrap();
    assert!(aSurface.is_u_rational() || aSurface.is_v_rational());
}

#[test]
fn NetworkSurface_IncompatibleRationalFamilyReportsStatus() {
    let aProfiles = bsplines(&[
        makeRationalQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.5, 0.0, 0.0), p(1.0, 0.0, 0.0), 0.5),
        makeWeightedLinearBSpline(p(0.0, 1.0, 0.0), p(1.0, 1.0, 0.0), 1.0),
    ]);
    let aGuides = bsplines(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    let aProfileParams = reals(&[0.0, 1.0]);
    let aGuideParams = reals(&[0.0, 1.0]);

    let aNetwork = unit_weight_network_of(
        (&aProfiles, &aGuides),
        (&aProfileParams, &aGuideParams),
        precision::confusion(),
        (false, false),
    );

    assert!(!aNetwork.is_done());
    assert_eq!(aNetwork.status(), NetworkSurfaceStatus::CurveCompatibilityFailed);
}

#[test]
fn NetworkSurface_RationalDegreeOverflowReportsStatus() {
    const aDegree: i32 = 9;
    let (aProfileList, aGuideList) = rational_line_network(aDegree);
    let aProfiles = bsplines(&aProfileList);
    let aGuides = bsplines(&aGuideList);
    let aProfileParams = reals(&[0.0, 0.5, 1.0]);
    let aGuideParams = reals(&[0.0, 0.5, 1.0]);

    let aNetwork = unit_weight_network_of(
        (&aProfiles, &aGuides),
        (&aProfileParams, &aGuideParams),
        precision::confusion(),
        (false, false),
    );

    assert!(!aNetwork.is_done());
    assert_eq!(aNetwork.status(), NetworkSurfaceStatus::RationalDegreeOverflow);
}

#[test]
fn RationalProfilesWithPolynomialGuidesProducesSurface() {
    let aProfiles = curves(&[
        makeRationalQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.5, 0.0, 0.0), p(1.0, 0.0, 0.0), 0.5),
        makeRationalQuadraticBSpline(p(0.0, 0.5, 0.0), p(0.5, 0.5, 0.0), p(1.0, 0.5, 0.0), 0.5),
        makeRationalQuadraticBSpline(p(0.0, 1.0, 0.0), p(0.5, 1.0, 0.0), p(1.0, 1.0, 0.0), 0.5),
    ]);
    let aGuides = curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(0.5, 0.0, 0.0), p(0.5, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.is_approximate());
    let aSurface = aGordon.surface().unwrap();
    assert!(aSurface.is_u_rational() || aSurface.is_v_rational());
    assert!(aGordon.report().max_profile_deviation <= precision::confusion());
    assert!(aGordon.report().max_guide_deviation <= precision::confusion());
}

#[test]
fn PolynomialProfilesWithRationalGuidesProducesSurface() {
    let aProfiles = curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0)),
        makeLinearBSpline(p(0.0, 0.5, 0.0), p(1.0, 0.5, 0.0)),
        makeLinearBSpline(p(0.0, 1.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    let aGuides = curves(&[
        makeRationalQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.0, 0.5, 0.0), p(0.0, 1.0, 0.0), 0.5),
        makeRationalQuadraticBSpline(p(0.5, 0.0, 0.0), p(0.5, 0.5, 0.0), p(0.5, 1.0, 0.0), 0.5),
        makeRationalQuadraticBSpline(p(1.0, 0.0, 0.0), p(1.0, 0.5, 0.0), p(1.0, 1.0, 0.0), 0.5),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.is_approximate());
    let aSurface = aGordon.surface().unwrap();
    assert!(aSurface.is_u_rational() || aSurface.is_v_rational());
    assert!(aGordon.report().max_profile_deviation <= precision::confusion());
    assert!(aGordon.report().max_guide_deviation <= precision::confusion());
}

#[test]
fn ApproximateFallbackCanRecoverRationalDegreeOverflow() {
    const aDegree: i32 = 9;
    let (aProfileList, aGuideList) = rational_line_network(aDegree);
    let aProfiles = curves(&aProfileList);
    let aGuides = curves(&aGuideList);

    let mut aGordon = GeomFillGordon::new();
    aGordon.init(&aProfiles, &aGuides, precision::confusion()).unwrap();
    aGordon.set_approximation_mode(GordonApproximationMode::AllowApproximateFallback);
    aGordon.perform().unwrap();

    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    assert!(aGordon.is_approximate());
    assert_eq!(aGordon.report().status, GordonStatus::Done);
    assert!(aGordon.report().is_approximate);
    assert_eq!(aGordon.report().failed_stage, GordonBuildStage::NotStarted);
    assert!(aGordon.report().max_approximation_deviation <= 0.001);
    assert!(!aGordon.surface().unwrap().is_null());

    const THE_NB_QUALITY_SAMPLES: usize = 48;
    let aSurface = aGordon.surface().unwrap();
    let ((aUMin, aUMax), (aVMin, aVMax)) = (u_range(&aSurface), v_range(&aSurface));
    let mut aMaxDeviation: f64 = 0.0;
    for aUIdx in 0..=THE_NB_QUALITY_SAMPLES {
        let aUParam = aUIdx as f64 / THE_NB_QUALITY_SAMPLES as f64;
        let aU = aUMin + aUParam * (aUMax - aUMin);
        for aVIdx in 0..=THE_NB_QUALITY_SAMPLES {
            let aVParam = aVIdx as f64 / THE_NB_QUALITY_SAMPLES as f64;
            let aV = aVMin + aVParam * (aVMax - aVMin);
            // `At(0)` is the zero-based access: the first curve.
            let aProfilePoint = aProfiles[1].value(aUParam).unwrap();
            let aGuidePoint = aGuides[1].value(aVParam).unwrap();
            let anExpectedPoint = p(aProfilePoint.x(), aGuidePoint.y(), 0.0);
            aMaxDeviation = aMaxDeviation.max(aSurface.value(aU, aV).unwrap().distance(&anExpectedPoint));
        }
    }
    // An interior Gordon point combines the independently approximated profile and guide skins.
    assert!(aMaxDeviation <= 0.002, "{aMaxDeviation}");
}

#[test]
fn ExactOnlyReportsRationalDegreeOverflow() {
    const aDegree: i32 = 9;
    let (aProfileList, aGuideList) = rational_line_network(aDegree);
    let aProfiles = curves(&aProfileList);
    let aGuides = curves(&aGuideList);

    let mut aGordon = GeomFillGordon::new();
    aGordon.init(&aProfiles, &aGuides, precision::confusion()).unwrap();
    assert_eq!(aGordon.get_approximation_mode(), GordonApproximationMode::ExactOnly);
    aGordon.perform().unwrap();

    assert!(!aGordon.is_done());
    assert!(!aGordon.is_approximate());
    assert_eq!(aGordon.status(), GordonStatus::RationalDegreeOverflow);
    assert_eq!(aGordon.report().status, GordonStatus::RationalDegreeOverflow);
    assert_eq!(aGordon.report().failed_stage, GordonBuildStage::ExactConstruction);
    assert!(!aGordon.report().is_approximate);
}

#[test]
fn ExactOnlyReparametrizesRationalCurves() {
    let (aProfiles, aGuides) = rational_quadratic_line_network();
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.is_approximate());
    assert_eq!(aGordon.status(), GordonStatus::Done);
    assert_eq!(aGordon.report().status, GordonStatus::Done);
    assert_eq!(aGordon.report().failed_stage, GordonBuildStage::NotStarted);
    assert!(!aGordon.report().is_approximate);
    assert_near(aGordon.report().max_reparametrization_deviation, 0.0, 1.0e-12);
}

#[test]
fn ApproximationModeUsesExactRationalReparametrizationWhenPossible() {
    let (aProfiles, aGuides) = rational_quadratic_line_network();

    let mut aGordon = GeomFillGordon::new();
    aGordon.init(&aProfiles, &aGuides, precision::confusion()).unwrap();
    aGordon.set_approximation_mode(GordonApproximationMode::AllowApproximateFallback);
    aGordon.perform().unwrap();

    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    assert!(!aGordon.is_approximate());
    assert_eq!(aGordon.report().status, GordonStatus::Done);
    assert!(!aGordon.report().is_approximate);
    assert_near(aGordon.report().max_reparametrization_deviation, 0.0, 1.0e-12);
    assert!(!aGordon.surface().unwrap().is_null());
}

// The report checks after each Perform of ExactRationalReparametrizationRepeatPerform.
fn assert_stable_exact_report(aGordon: &GeomFillGordon) {
    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    assert!(!aGordon.is_approximate());
    assert_eq!(aGordon.report().status, GordonStatus::Done);
    assert_eq!(aGordon.report().failed_stage, GordonBuildStage::NotStarted);
    assert!(!aGordon.report().is_approximate);
    assert_near(aGordon.report().max_contact_gap, 0.0, 1.0e-12);
    assert_near(aGordon.report().max_reparametrization_deviation, 0.0, 1.0e-12);
    assert_near(aGordon.report().max_approximation_deviation, 0.0, 1.0e-12);
}

#[test]
fn ExactRationalReparametrizationRepeatPerformKeepsReportStable() {
    let (aProfiles, aGuides) = rational_quadratic_line_network();

    let mut aGordon = GeomFillGordon::new();
    aGordon.init(&aProfiles, &aGuides, precision::confusion()).unwrap();
    aGordon.set_approximation_mode(GordonApproximationMode::AllowApproximateFallback);

    aGordon.perform().unwrap();
    assert_stable_exact_report(&aGordon);
    let aFirstCenter = aGordon.surface().unwrap().value(0.5, 0.5).unwrap();

    aGordon.perform().unwrap();
    assert_stable_exact_report(&aGordon);
    let aSecondCenter = aGordon.surface().unwrap().value(0.5, 0.5).unwrap();
    assert_near(aFirstCenter.distance(&aSecondCenter), 0.0, precision::confusion());
}

#[test]
fn NetworkSurface_VClosedNetwork_ProducesVPeriodicSurface() {
    let aProfiles = bsplines(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(2.0, 0.0, 0.0)),
        makeLinearBSpline(p(0.0, 1.0, 0.0), p(2.0, 1.0, 1.0)),
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(2.0, 0.0, 0.0)),
    ]);
    let aGuides = bsplines(&[
        makeQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0), p(0.0, 0.0, 0.0)),
        makeQuadraticBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0), p(1.0, 0.0, 0.0)),
        makeQuadraticBSpline(p(2.0, 0.0, 0.0), p(2.0, 1.0, 0.0), p(2.0, 0.0, 0.0)),
    ]);
    let aProfileParams = reals(&[0.0, 0.5, 1.0]);
    let aGuideParams = reals(&[0.0, 0.5, 1.0]);

    let aNetwork = unit_weight_network_of(
        (&aProfiles, &aGuides),
        (&aProfileParams, &aGuideParams),
        precision::confusion(),
        (false, true),
    );

    assert!(aNetwork.is_done());

    let aSurf = aNetwork.surface().unwrap();
    assert!(!aSurf.is_null());
    assert!(aSurf.is_v_periodic());
}

// Init + Perform of the analytic network, which must succeed exactly.
fn exact_gordon_of(aNetwork: &AnalyticNetwork, theTolerance: f64) -> GeomFillGordon {
    let aGordon = gordon_of(&aNetwork.Profiles, &aNetwork.Guides, theTolerance);
    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    assert!(!aGordon.is_approximate());
    aGordon
}

#[test]
fn DenseWavyFiveByFiveNetwork_InterpolatesAllInputCurves() {
    let aNetwork = AnalyticNetwork::new(5, 5, wavyHeight);
    let aGordon = exact_gordon_of(&aNetwork, precision::confusion());
    verifyUniformNetworkInterpolation(&aGordon.surface().unwrap(), &aNetwork, precision::confusion());
}

#[test]
fn DenseSaddleSevenByFourNetwork_InterpolatesAllInputCurves() {
    let aNetwork = AnalyticNetwork::new(7, 4, saddleHeight);
    let aGordon = exact_gordon_of(&aNetwork, precision::confusion());
    verifyUniformNetworkInterpolation(&aGordon.surface().unwrap(), &aNetwork, precision::confusion());
}

#[test]
fn DenseRippleFourBySevenNetwork_InterpolatesAllInputCurves() {
    let aNetwork = AnalyticNetwork::new(4, 7, rippleHeight);
    let aGordon = exact_gordon_of(&aNetwork, precision::confusion());
    verifyUniformNetworkInterpolation(&aGordon.surface().unwrap(), &aNetwork, precision::confusion());
}

#[test]
fn DenseWavyEightByEightNetwork_InterpolatesAllInputCurves() {
    let aNetwork = AnalyticNetwork::new(8, 8, wavyHeight);
    let aGordon = exact_gordon_of(&aNetwork, precision::confusion());
    verifyUniformNetworkInterpolation(&aGordon.surface().unwrap(), &aNetwork, precision::confusion());
}

#[test]
fn TranslatedScaledSixByFiveNetwork_InterpolatesAllInputCurves() {
    let aNetwork = AnalyticNetwork::with_frame(6, 5, rippleHeight, Xyz::new(1.0e5, -2.0e5, 3.0e5), 250.0);
    let aGordon = exact_gordon_of(&aNetwork, 1.0e-5);
    verifyUniformNetworkInterpolation(&aGordon.surface().unwrap(), &aNetwork, 1.0e-5);
}

#[test]
fn SmallScaledFiveBySixNetwork_InterpolatesAllInputCurves() {
    let aNetwork = AnalyticNetwork::with_frame(5, 6, saddleHeight, Xyz::new(0.0, 0.0, 0.0), 0.01);
    let aGordon = exact_gordon_of(&aNetwork, precision::confusion());
    verifyUniformNetworkInterpolation(&aGordon.surface().unwrap(), &aNetwork, precision::confusion());
}

// The largest distance between two surfaces on a 25 x 25 grid of `[0, 1]^2`.
fn assert_same_surface(aFirst: &Handle<GeomBSplineSurface>, aSecond: &Handle<GeomBSplineSurface>) {
    const THE_NB_SAMPLES: usize = 24;
    for aUIdx in 0..=THE_NB_SAMPLES {
        for aVIdx in 0..=THE_NB_SAMPLES {
            let aU = aUIdx as f64 / THE_NB_SAMPLES as f64;
            let aV = aVIdx as f64 / THE_NB_SAMPLES as f64;
            let d = aFirst.value(aU, aV).unwrap().distance(&aSecond.value(aU, aV).unwrap());
            assert!(d <= precision::confusion(), "{d} at ({aU}, {aV})");
        }
    }
}

#[test]
fn DenseWavyNetwork_ParallelConstructionMatchesSerialQuality() {
    let aNetwork = AnalyticNetwork::new(6, 6, wavyHeight);
    let aSerial = gordon_of(&aNetwork.Profiles, &aNetwork.Guides, precision::confusion());
    assert!(aSerial.is_done(), "{:?}", aSerial.status());

    let mut aParallel = GeomFillGordon::new();
    aParallel.set_parallel_mode(true);
    aParallel.init(&aNetwork.Profiles, &aNetwork.Guides, precision::confusion()).unwrap();
    aParallel.perform().unwrap();
    assert!(aParallel.is_done(), "{:?}", aParallel.status());

    verifyUniformNetworkInterpolation(&aSerial.surface().unwrap(), &aNetwork, precision::confusion());
    verifyUniformNetworkInterpolation(&aParallel.surface().unwrap(), &aNetwork, precision::confusion());
    assert_same_surface(&aSerial.surface().unwrap(), &aParallel.surface().unwrap());
}

#[test]
fn DenseSaddleNetwork_RepeatedPerformMaintainsQuality() {
    let aNetwork = AnalyticNetwork::new(6, 5, saddleHeight);
    let mut aGordon = gordon_of(&aNetwork.Profiles, &aNetwork.Guides, precision::confusion());
    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    let aFirstSurface = aGordon.surface().unwrap();

    aGordon.perform().unwrap();
    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    verifyUniformNetworkInterpolation(&aGordon.surface().unwrap(), &aNetwork, precision::confusion());
    assert_same_surface(&aFirstSurface, &aGordon.surface().unwrap());
}

#[test]
fn DenseRippleNineByThreeNetwork_InterpolatesAllInputCurves() {
    let aNetwork = AnalyticNetwork::new(9, 3, rippleHeight);
    let aGordon = exact_gordon_of(&aNetwork, precision::confusion());
    verifyUniformNetworkInterpolation(&aGordon.surface().unwrap(), &aNetwork, precision::confusion());
}

#[test]
fn DenseWavyThreeByNineNetwork_InterpolatesAllInputCurves() {
    let aNetwork = AnalyticNetwork::new(3, 9, wavyHeight);
    let aGordon = exact_gordon_of(&aNetwork, precision::confusion());
    verifyUniformNetworkInterpolation(&aGordon.surface().unwrap(), &aNetwork, precision::confusion());
}

#[test]
fn ClosedBSplineProfilesWithInteriorGuides_ProducePeriodicSurface() {
    let aProfiles = curves(&[makeClosedBSplineProfile(0.0, 1.0, 1.0), makeClosedBSplineProfile(1.0, 1.15, 0.85)]);
    let aFirst: Handle<GeomBSplineCurve> = aProfiles[1].cast();
    assert!(!aFirst.is_periodic());
    let aGap = aProfiles[1]
        .value(aProfiles[1].first_parameter())
        .unwrap()
        .distance(&aProfiles[1].value(aProfiles[1].last_parameter()).unwrap());
    assert!(aGap <= precision::confusion());

    let aGuideParameters = reals(&[0.12, 0.34, 0.59, 0.83]);
    let aGuideList: Vec<Handle<GeomBSplineCurve>> = (aGuideParameters.lower()..=aGuideParameters.upper())
        .map(|aGuideIdx| {
            let aParameter = aGuideParameters[aGuideIdx];
            makeLinearBSpline(aProfiles[1].value(aParameter).unwrap(), aProfiles[2].value(aParameter).unwrap())
        })
        .collect();
    let aGuides = curves(&aGuideList);

    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());
    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    assert!(!aGordon.is_approximate());
    assert!(aGordon.surface().unwrap().is_u_periodic());
    assert!(aGordon.report().max_profile_deviation <= precision::confusion());
    assert!(aGordon.report().max_guide_deviation <= precision::confusion());
}

#[test]
fn NetworkSurface_ClosedNetworkWithNonUnitUParameters_ProducesUPeriodicSurface() {
    let aProfiles = bsplines(&[
        makeQuadraticBSplineOn(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(0.0, 0.0, 0.0), 2.0, 5.0),
        makeQuadraticBSplineOn(p(0.0, 1.0, 0.0), p(1.0, 1.0, 1.0), p(0.0, 1.0, 0.0), 2.0, 5.0),
    ]);

    let aGuideParameters = reals(&[2.0, 3.5]);
    let aGuideList: Vec<Handle<GeomBSplineCurve>> = (aGuideParameters.lower()..=aGuideParameters.upper())
        .map(|aGuideIdx| {
            let aParameter = aGuideParameters[aGuideIdx];
            makeLinearBSpline(aProfiles[1].value(aParameter).unwrap(), aProfiles[2].value(aParameter).unwrap())
        })
        .collect();
    let aGuides = bsplines(&aGuideList);

    let aProfileParameters = reals(&[0.0, 1.0]);

    let aNetwork = unit_weight_network_of(
        (&aProfiles, &aGuides),
        (&aProfileParameters, &aGuideParameters),
        precision::confusion(),
        (true, false),
    );

    assert!(aNetwork.is_done(), "{:?}", aNetwork.status());
    assert!(aNetwork.surface().unwrap().is_u_periodic());
}
