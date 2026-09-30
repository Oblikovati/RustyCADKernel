// GeomFill_Gordon tests (high-level, automatic intersection detection), part 1.

const X: (f64, f64, f64) = (1.0, 0.0, 0.0);
const Y: (f64, f64, f64) = (0.0, 1.0, 0.0);

// The unit-square network of trimmed lines of SimpleLineNetwork.
fn unit_line_network() -> CurveNetwork {
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 1.0, 0.0), X, 0.0, 1.0),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(1.0, 0.0, 0.0), Y, 0.0, 1.0),
    ]);
    (aProfiles, aGuides)
}

// The quadratic-bump profiles of ParallelMode and CurvedBSplineNetwork.
fn bump_network() -> CurveNetwork {
    let aProfiles = curves(&[
        makeQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.5, 0.0, 0.4), p(1.0, 0.0, 0.0)),
        makeQuadraticBSpline(p(0.0, 0.5, 0.0), p(0.5, 0.5, 0.6), p(1.0, 0.5, 0.0)),
        makeQuadraticBSpline(p(0.0, 1.0, 0.0), p(0.5, 1.0, 0.4), p(1.0, 1.0, 0.0)),
    ]);
    let aGuides = curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    (aProfiles, aGuides)
}

#[test]
fn GeomFill_Gordon__SimpleLineNetwork_ProducesValidSurface() {
    let (aProfiles, aGuides) = unit_line_network();
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert_eq!(aGordon.status(), GordonStatus::Done);
    assert_eq!(aGordon.report().status, GordonStatus::Done);
    assert_eq!(aGordon.report().failed_stage, GordonBuildStage::NotStarted);
    assert!(!aGordon.report().is_approximate);
    assert!(aGordon.report().max_contact_gap <= precision::confusion());
    assert!(aGordon.report().max_profile_deviation <= 1.0e-7);
    assert!(aGordon.report().max_guide_deviation <= 1.0e-7);

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    verifyPointOnSurface(&aSurf, 0.0, 0.0, p(0.0, 0.0, 0.0), 1.0e-3);
    verifyPointOnSurface(&aSurf, 1.0, 0.0, p(1.0, 0.0, 0.0), 1.0e-3);
    verifyPointOnSurface(&aSurf, 0.0, 1.0, p(0.0, 1.0, 0.0), 1.0e-3);
    verifyPointOnSurface(&aSurf, 1.0, 1.0, p(1.0, 1.0, 0.0), 1.0e-3);
}

#[test]
fn GeomFill_Gordon__EmptyInput_NotDone() {
    let aOneProfile = curve_array(vec![trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 1.0)]);
    let aOneGuide = curve_array(vec![trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 1.0)]);
    let aGordon = gordon_of(&aOneProfile, &aOneGuide, precision::confusion());

    assert!(!aGordon.is_done());
    assert_eq!(aGordon.status(), GordonStatus::InvalidInput);
}

#[test]
fn GeomFill_Gordon__SingleProfileOrGuide_NotDone() {
    let aProfiles = curve_array(vec![trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 1.0)]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(1.0, 0.0, 0.0), Y, 0.0, 1.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(!aGordon.is_done());
    assert_eq!(aGordon.status(), GordonStatus::InvalidInput);
}

#[test]
fn GeomFill_Gordon__NotDone_BeforePerform() {
    let aGordon = GeomFillGordon::new();
    assert!(!aGordon.is_done());
    assert_eq!(aGordon.status(), GordonStatus::NotStarted);
    assert_eq!(aGordon.report().status, GordonStatus::NotStarted);
    assert_eq!(aGordon.report().failed_stage, GordonBuildStage::NotStarted);
    assert!(!aGordon.report().is_approximate);
    assert_eq!(surface_error(&aGordon), "StdFail_NotDone");
}

#[test]
fn GeomFill_Gordon__ReinitResetsReportAndSurface() {
    let (aProfiles, aGuides) = unit_line_network();
    let mut aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert_eq!(aGordon.report().status, GordonStatus::Done);
    assert!(!aGordon.surface().unwrap().is_null());

    let aOneProfile = curve_array(vec![trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 1.0)]);
    let aOneGuide = curve_array(vec![trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 1.0)]);

    aGordon.init(&aOneProfile, &aOneGuide, precision::confusion()).unwrap();
    assert!(!aGordon.is_done());
    assert_eq!(aGordon.status(), GordonStatus::NotStarted);
    assert_eq!(aGordon.report().status, GordonStatus::NotStarted);
    assert_eq!(aGordon.report().failed_stage, GordonBuildStage::NotStarted);
    assert!(!aGordon.report().is_approximate);
    assert_near(aGordon.report().max_contact_gap, 0.0, 1.0e-12);
    assert_near(aGordon.report().max_reparametrization_deviation, 0.0, 1.0e-12);
    assert_near(aGordon.report().max_profile_deviation, 0.0, 1.0e-12);
    assert_near(aGordon.report().max_guide_deviation, 0.0, 1.0e-12);
    assert_near(aGordon.report().max_approximation_deviation, 0.0, 1.0e-12);
    assert_eq!(surface_error(&aGordon), "StdFail_NotDone");

    aGordon.perform().unwrap();
    assert!(!aGordon.is_done());
    assert_eq!(aGordon.status(), GordonStatus::InvalidInput);
    assert_eq!(aGordon.report().status, GordonStatus::InvalidInput);
    assert_eq!(aGordon.report().failed_stage, GordonBuildStage::InputConversion);
}

#[test]
fn GeomFill_Gordon__DisjointNetworkReportsContactDiscoveryFailure() {
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 1.0, 0.0), X, 0.0, 1.0),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(2.0, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(3.0, 0.0, 0.0), Y, 0.0, 1.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(!aGordon.is_done());
    assert!(!aGordon.is_approximate());
    assert_eq!(aGordon.status(), GordonStatus::IntersectionFailed);
    assert_eq!(aGordon.report().status, GordonStatus::IntersectionFailed);
    assert_eq!(aGordon.report().failed_stage, GordonBuildStage::ContactDiscovery);
    assert!(!aGordon.report().is_approximate);
    assert_near(aGordon.report().max_contact_gap, 0.0, 1.0e-12);
    assert_near(aGordon.report().max_reparametrization_deviation, 0.0, 1.0e-12);
    assert_eq!(surface_error(&aGordon), "StdFail_NotDone");
}

#[test]
fn GeomFill_Gordon__ParallelMode_DefaultSingleThreadAndMatchesParallelResult() {
    let (aProfiles, aGuides) = bump_network();

    let mut aSingleThread = GeomFillGordon::new();
    assert!(!aSingleThread.is_parallel_mode());
    aSingleThread.init(&aProfiles, &aGuides, precision::confusion()).unwrap();
    aSingleThread.perform().unwrap();
    assert!(aSingleThread.is_done());

    let mut aParallel = GeomFillGordon::new();
    aParallel.set_parallel_mode(true);
    assert!(aParallel.is_parallel_mode());
    aParallel.init(&aProfiles, &aGuides, precision::confusion()).unwrap();
    aParallel.perform().unwrap();
    assert!(aParallel.is_done());

    let aSurfSingle = aSingleThread.surface().unwrap();
    let aSurfPar = aParallel.surface().unwrap();

    let aMidSingle = aSurfSingle.value(0.5, 0.5).unwrap();
    let aMidPar = aSurfPar.value(0.5, 0.5).unwrap();
    assert!(aMidSingle.distance(&aMidPar) < 1.0e-9);
}

#[test]
fn GeomFill_Gordon__CurvedBSplineNetwork_ProducesValidSurface() {
    // 3 quadratic profiles with Z-curvature + 2 linear guides.
    let (aProfiles, aGuides) = bump_network();
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    let ((u1, u2), (v1, v2)) = (u_range(&aSurf), v_range(&aSurf));
    let aMidPt = aSurf.value(0.5 * (u1 + u2), 0.5 * (v1 + v2)).unwrap();
    assert_near(aMidPt.z(), 0.3, 1.0e-6); // Surface should have Z-bump at center
}

#[test]
fn GeomFill_Gordon__NonAffineProfileReparametrization_PreservesProfilesAndGuideContinuity() {
    let aProfiles = curves(&[
        makeCubicInterpBSpline(p(0.0, 0.0, 0.0), p(0.25, 0.0, 0.4), p(0.75, 0.0, -0.3), p(1.0, 0.0, 0.0)),
        makeCubicInterpBSpline(p(0.0, 0.5, 0.0), p(0.25, 0.5, 0.2), p(0.75, 0.5, -0.5), p(1.0, 0.5, 0.0)),
        makeCubicInterpBSpline(p(0.0, 1.0, 0.0), p(0.25, 1.0, 0.5), p(0.75, 1.0, -0.2), p(1.0, 1.0, 0.0)),
    ]);

    let aGuideCurve = interpolated(
        vec![aProfiles[1].value(0.4).unwrap(), aProfiles[2].value(0.5).unwrap(), aProfiles[3].value(0.6).unwrap()],
        Some(vec![0.0, 0.5, 1.0]),
        false,
    );

    let aGuides = curves(&[
        makeLinearBSpline(aProfiles[1].value(0.0).unwrap(), aProfiles[3].value(0.0).unwrap()),
        aGuideCurve,
        makeLinearBSpline(aProfiles[1].value(1.0).unwrap(), aProfiles[3].value(1.0).unwrap()),
    ]);

    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    let aSurface = aGordon.surface().unwrap();
    assert!(aGordon.report().max_profile_deviation <= precision::confusion());
    assert!(aGordon.report().max_guide_deviation <= precision::confusion());
    let ((u1, u2), (v1, v2)) = (u_range(&aSurface), v_range(&aSurface));
    let aMiddleProfileV = 0.5 * (v1 + v2);
    const THE_NB_SAMPLES: i32 = 32;
    for aSampleIdx in 0..=THE_NB_SAMPLES {
        let aRatio = f64::from(aSampleIdx) / f64::from(THE_NB_SAMPLES);
        let aU = u1 + aRatio * (u2 - u1);
        let aProfile1Param = if aRatio <= 0.5 { 0.8 * aRatio } else { 1.2 * aRatio - 0.2 };
        let aProfile3Param = if aRatio <= 0.5 { 1.2 * aRatio } else { 0.2 + 0.8 * aRatio };

        let d1 = aSurface.value(aU, v1).unwrap().distance(&aProfiles[1].value(aProfile1Param).unwrap());
        assert!(d1 <= precision::confusion(), "{d1}");
        let d2 = aSurface.value(aU, aMiddleProfileV).unwrap().distance(&aProfiles[2].value(aRatio).unwrap());
        assert!(d2 <= precision::confusion(), "{d2}");
        let d3 = aSurface.value(aU, v2).unwrap().distance(&aProfiles[3].value(aProfile3Param).unwrap());
        assert!(d3 <= precision::confusion(), "{d3}");
    }

    let aGuideU = 0.5 * (u1 + u2);
    for aSampleIdx in 0..=THE_NB_SAMPLES {
        let aRatio = f64::from(aSampleIdx) / f64::from(THE_NB_SAMPLES);
        let aV = v1 + aRatio * (v2 - v1);
        let d = aSurface.value(aGuideU, aV).unwrap().distance(&aGuides[2].value(aRatio).unwrap());
        assert!(d <= precision::confusion(), "{d}");
    }
}

#[test]
fn GeomFill_Gordon__MixedCurveTypes_ProducesValidSurface() {
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 2.0),
        trimmed_line(p(0.0, 2.0, 0.0), X, 0.0, 2.0),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 2.0),
        trimmed_line(p(2.0, 0.0, 0.0), Y, 0.0, 2.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.surface().unwrap().is_null());

    let aSurf = aGordon.surface().unwrap();
    verifyPointOnSurface(&aSurf, aSurf.u_knot(1).unwrap(), aSurf.v_knot(1).unwrap(), p(0.0, 0.0, 0.0), 0.01);
}

#[test]
fn GeomFill_Gordon__PeriodicBSplineProfiles_AreExpandedBeforeConstruction() {
    let aProfiles = curves(&[makePeriodicProfile(0.0), makePeriodicProfile(0.5), makePeriodicProfile(1.0)]);
    let aFirst: Handle<GeomBSplineCurve> = aProfiles[1].cast();
    assert!(aFirst.is_periodic());

    let aGuides = curves(&[
        makeLinearBSpline(aProfiles[1].value(0.0).unwrap(), aProfiles[3].value(0.0).unwrap()),
        makeLinearBSpline(aProfiles[1].value(0.5).unwrap(), aProfiles[3].value(0.5).unwrap()),
        makeLinearBSpline(aProfiles[1].value(1.0).unwrap(), aProfiles[3].value(1.0).unwrap()),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.surface().unwrap().is_null());
    assert!(aGordon.surface().unwrap().is_u_periodic());
}

#[test]
fn GeomFill_Gordon__PeriodicBSplineGuides_AreExpandedBeforeConstruction() {
    let aGuides = curves(&[makePeriodicGuide(0.0), makePeriodicGuide(0.5), makePeriodicGuide(1.0)]);
    let aFirst: Handle<GeomBSplineCurve> = aGuides[1].cast();
    assert!(aFirst.is_periodic());

    let aProfiles = curves(&[
        makeLinearBSpline(aGuides[1].value(0.0).unwrap(), aGuides[3].value(0.0).unwrap()),
        makeLinearBSpline(aGuides[1].value(0.5).unwrap(), aGuides[3].value(0.5).unwrap()),
        makeLinearBSpline(aGuides[1].value(1.0).unwrap(), aGuides[3].value(1.0).unwrap()),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.surface().unwrap().is_null());
    assert!(aGordon.surface().unwrap().is_v_periodic());
}

#[test]
fn GeomFill_Gordon__InterpolatedPeriodicProfilesWithAlignedSeams_InterpolateWithoutDeviation() {
    let aProfiles = curves(&[makeInterpolatedPeriodicProfile(0.0, 0), makeInterpolatedPeriodicProfile(1.0, 2)]);

    let aFirstProfile: Handle<GeomBSplineCurve> = aProfiles[1].cast();
    let aSecondProfile: Handle<GeomBSplineCurve> = aProfiles[2].cast();
    let aFirstSeam = aFirstProfile.value(aFirstProfile.first_parameter()).unwrap();
    let aSeamProjection =
        ProjectPointOnCurve::with_curve(p(aFirstSeam.x(), 1.0, aFirstSeam.z()), &aSecondProfile.cast()).unwrap();
    assert!(aSeamProjection.nb_points() > 0);
    aSecondProfile
        .set_origin_parameter(aSeamProjection.lower_distance_parameter().unwrap(), precision::confusion())
        .unwrap();
    let aFirstProfileMiddle = 0.5 * (aFirstProfile.first_parameter() + aFirstProfile.last_parameter());
    let aSecondProfileMiddle = 0.5 * (aSecondProfile.first_parameter() + aSecondProfile.last_parameter());
    let aSecondSeam = aSecondProfile.value(aSecondProfile.first_parameter()).unwrap();
    let aFirstMiddle = aFirstProfile.value(aFirstProfileMiddle).unwrap();
    let aSecondMiddle = aSecondProfile.value(aSecondProfileMiddle).unwrap();
    assert!(aFirstSeam.distance(&p(aSecondSeam.x(), aFirstSeam.y(), aSecondSeam.z())) <= precision::confusion());
    assert!(
        aFirstMiddle.distance(&p(aSecondMiddle.x(), aFirstMiddle.y(), aSecondMiddle.z())) <= precision::confusion()
    );

    let aFirstStart = aFirstProfile.value(aFirstProfile.first_parameter()).unwrap();
    let aSecondStart = aSecondProfile.value(aSecondProfile.first_parameter()).unwrap();
    let aGuides = curves(&[
        makeLinearBSpline(aFirstStart, aSecondStart),
        makeLinearBSpline(aFirstMiddle, aSecondMiddle),
        makeLinearBSpline(aFirstStart, aSecondStart),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    let aSurface = aGordon.surface().unwrap();
    assert!(aSurface.is_u_periodic());
    assert!(aGordon.report().max_profile_deviation <= precision::confusion());
    assert!(aGordon.report().max_guide_deviation <= precision::confusion());

    let ((u1, u2), (v1, v2)) = (u_range(&aSurface), v_range(&aSurface));
    let aMiddleV = 0.5 * (v1 + v2);
    let aFirstSeamD1 = aSurface.eval_d1(u1, aMiddleV).unwrap();
    let aLastSeamD1 = aSurface.eval_d1(u2, aMiddleV).unwrap();
    assert!(aFirstSeamD1.d1u.angle(aLastSeamD1.d1u).unwrap() <= precision::angular());

    const THE_NB_SAMPLES: i32 = 16;
    for aSampleIdx in 0..=THE_NB_SAMPLES {
        let aRatio = f64::from(aSampleIdx) / f64::from(THE_NB_SAMPLES);
        let aU = u1 + aRatio * (u2 - u1);
        let aFirstProfileParam =
            aFirstProfile.first_parameter() + aRatio * (aFirstProfile.last_parameter() - aFirstProfile.first_parameter());
        let aSecondProfileParam = aSecondProfile.first_parameter()
            + aRatio * (aSecondProfile.last_parameter() - aSecondProfile.first_parameter());
        let d1 = aSurface.value(aU, v1).unwrap().distance(&aFirstProfile.value(aFirstProfileParam).unwrap());
        assert!(d1 <= precision::confusion(), "{d1}");
        let d2 = aSurface.value(aU, v2).unwrap().distance(&aSecondProfile.value(aSecondProfileParam).unwrap());
        assert!(d2 <= precision::confusion(), "{d2}");
    }
}

#[test]
fn GeomFill_Gordon__MultipleIntersections_SelectsMonotoneBranch() {
    let aProfiles = curves(&[makeMeanderingProfile(0.0), makeMeanderingProfile(0.5), makeMeanderingProfile(1.0)]);
    let aGuides = curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(0.5, 0.0, 0.0), p(0.5, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    let mut aGordon = gordon_of(&aProfiles, &aGuides, 1.0e-5);

    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    assert!(!aGordon.surface().unwrap().is_null());
    assert!(aGordon.report().max_profile_deviation <= 1.0e-5);
    assert!(aGordon.report().max_guide_deviation <= 1.0e-5);
    let aSurf = aGordon.surface().unwrap();
    verifyPointOnSurface(&aSurf, 0.0, 0.0, p(0.0, 0.0, 0.0), 1.0e-3);
    verifyPointOnSurface(&aSurf, 1.0, 0.0, p(1.0, 0.0, 0.0), 1.0e-3);
    verifyPointOnSurface(&aSurf, 0.0, 1.0, p(0.0, 1.0, 0.0), 1.0e-3);
    verifyPointOnSurface(&aSurf, 1.0, 1.0, p(1.0, 1.0, 0.0), 1.0e-3);

    let aFirstCenter = aSurf.value(0.5, 0.5).unwrap();
    aGordon.perform().unwrap();
    assert!(aGordon.is_done(), "{:?}", aGordon.status());
    assert!(aFirstCenter.distance(&aGordon.surface().unwrap().value(0.5, 0.5).unwrap()) <= precision::confusion());
}

#[test]
fn GeomFill_Gordon__FourByThreeGrid_NonUniformParams() {
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 0.2, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 0.7, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 1.0, 0.0), X, 0.0, 1.0),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(0.3, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(1.0, 0.0, 0.0), Y, 0.0, 1.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.surface().unwrap().is_null());
}

#[test]
fn GeomFill_Gordon__SurfaceAlongInputCurves_IsAccurate() {
    let (aProfiles, aGuides) = unit_line_network();
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    let aSurf = aGordon.surface().unwrap();
    let ((aUMin, aUMax), (aVMin, aVMax)) = (u_range(&aSurf), v_range(&aSurf));

    const THE_NB_SAMPLES: i32 = 20;

    // Along first profile (V = VMin): Y should be 0, Z should be 0.
    for i in 0..=THE_NB_SAMPLES {
        let aU = aUMin + f64::from(i) * (aUMax - aUMin) / f64::from(THE_NB_SAMPLES);
        let aSurfPt = aSurf.value(aU, aVMin).unwrap();
        assert_near(aSurfPt.y(), 0.0, 0.01);
        assert_near(aSurfPt.z(), 0.0, 0.01);
    }

    // Along last profile (V = VMax): Y should be 1.
    for i in 0..=THE_NB_SAMPLES {
        let aU = aUMin + f64::from(i) * (aUMax - aUMin) / f64::from(THE_NB_SAMPLES);
        let aSurfPt = aSurf.value(aU, aVMax).unwrap();
        assert_near(aSurfPt.y(), 1.0, 0.01);
        assert_near(aSurfPt.z(), 0.0, 0.01);
    }
}

#[test]
fn GeomFill_Gordon__ReversedCurveNetwork_ProducesValidSurface() {
    // Reversed profile and guide directions.
    let aProfiles = curve_array(vec![
        trimmed_line(p(1.0, 0.0, 0.0), (-1.0, 0.0, 0.0), 0.0, 1.0),
        trimmed_line(p(0.0, 1.0, 0.0), X, 0.0, 1.0),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(1.0, 1.0, 0.0), (0.0, -1.0, 0.0), 0.0, 1.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());
    let ((aUMin, aUMax), (aVMin, aVMax)) = (u_range(&aSurf), v_range(&aSurf));

    // All corners should span unit square despite reversed input.
    let aCorners = [
        aSurf.value(aUMin, aVMin).unwrap(),
        aSurf.value(aUMax, aVMin).unwrap(),
        aSurf.value(aUMin, aVMax).unwrap(),
        aSurf.value(aUMax, aVMax).unwrap(),
    ];

    let (mut aXMin, mut aXMax) = (f64::MAX, -f64::MAX);
    let (mut aYMin, mut aYMax) = (f64::MAX, -f64::MAX);
    for aCorner in &aCorners {
        assert_near(aCorner.z(), 0.0, 0.01);
        aXMin = aXMin.min(aCorner.x());
        aXMax = aXMax.max(aCorner.x());
        aYMin = aYMin.min(aCorner.y());
        aYMax = aYMax.max(aCorner.y());
    }
    assert_near(aXMin, 0.0, 0.01);
    assert_near(aXMax, 1.0, 0.01);
    assert_near(aYMin, 0.0, 0.01);
    assert_near(aYMax, 1.0, 0.01);
}

#[test]
fn GeomFill_Gordon__CubicInterpolatedNetwork_ProducesValidSurface() {
    // Cubic interpolated profiles (S-shaped in Z) + linear guides.
    let aProfiles = curves(&[
        makeCubicInterpBSpline(p(0.0, 0.0, 0.0), p(0.33, 0.0, 0.3), p(0.66, 0.0, -0.3), p(1.0, 0.0, 0.0)),
        makeCubicInterpBSpline(p(0.0, 0.5, 0.0), p(0.33, 0.5, 0.5), p(0.66, 0.5, -0.5), p(1.0, 0.5, 0.0)),
        makeCubicInterpBSpline(p(0.0, 1.0, 0.0), p(0.33, 1.0, 0.3), p(0.66, 1.0, -0.3), p(1.0, 1.0, 0.0)),
    ]);
    let aGuides = curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, 0.01);

    assert!(aGordon.is_done());
    assert!(!aGordon.surface().unwrap().is_null());

    // Surface should have non-trivial Z variation.
    let aSurf = aGordon.surface().unwrap();
    let ((u1, u2), (v1, v2)) = (u_range(&aSurf), v_range(&aSurf));
    let aUMid = 0.5 * (u1 + u2);
    let aVMid = 0.5 * (v1 + v2);

    // Verify some Z variation exists (the S-curve shape).
    let aP1 = aSurf.value(aUMid * 0.5, aVMid).unwrap();
    let aP2 = aSurf.value(aUMid * 1.5, aVMid).unwrap();
    assert_near(aP1.z(), 0.6328125, 1.0e-9); // S-curve first lobe should be preserved
    assert_near(aP2.z(), -0.6328125, 1.0e-9); // S-curve second lobe should be preserved
}

#[test]
fn GeomFill_Gordon__SinusoidalProfiles_SurfacePreservesShape() {
    // Sinusoidal profiles at different Y positions + linear guides.
    let aProfiles = curves(&[
        makeSineBSpline(0.0, 1.0, 0.0, 0.3),
        makeSineBSpline(0.0, 1.0, 0.5, 0.5),
        makeSineBSpline(0.0, 1.0, 1.0, 0.3),
    ]);
    let aGuides = curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, 0.01);

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    // At the center of the surface (U=0.5), the sine peak should be maximum.
    let ((u1, u2), (v1, v2)) = (u_range(&aSurf), v_range(&aSurf));
    let aMidPt = aSurf.value(0.5 * (u1 + u2), 0.5 * (v1 + v2)).unwrap();
    assert_near(aMidPt.z(), 0.5, 0.05); // Center should preserve middle sine amplitude
}

#[test]
fn GeomFill_Gordon__ThreeByThreeGrid_SurfaceInterpolatesAllCurves() {
    // 3 profiles + 3 guides. Check that the Gordon surface interpolates all
    // input curves by dense sampling along each one.
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 0.5, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 1.0, 0.0), X, 0.0, 1.0),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(0.5, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(1.0, 0.0, 0.0), Y, 0.0, 1.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    let ((aUMin, aUMax), (aVMin, aVMax)) = (u_range(&aSurf), v_range(&aSurf));
    const THE_NB: i32 = 10;
    let step = |i: i32, a: f64, b: f64| a + f64::from(i) * (b - a) / f64::from(THE_NB);

    // Profile 1 at V=VMin: Y~0.
    for i in 0..=THE_NB {
        assert_near(aSurf.value(step(i, aUMin, aUMax), aVMin).unwrap().y(), 0.0, 0.01);
    }
    // Profile 3 at V=VMax: Y~1.
    for i in 0..=THE_NB {
        assert_near(aSurf.value(step(i, aUMin, aUMax), aVMax).unwrap().y(), 1.0, 0.01);
    }
    // Guide 1 at U=UMin: X~0.
    for i in 0..=THE_NB {
        assert_near(aSurf.value(aUMin, step(i, aVMin, aVMax)).unwrap().x(), 0.0, 0.01);
    }
    // Guide 3 at U=UMax: X~1.
    for i in 0..=THE_NB {
        assert_near(aSurf.value(aUMax, step(i, aVMin, aVMax)).unwrap().x(), 1.0, 0.01);
    }
}

// The square network of trimmed lines of side `aScale` of the ScaledGeometry tests.
fn scaled_line_network(
    aScale: f64,
) -> CurveNetwork {
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, aScale),
        trimmed_line(p(0.0, aScale, 0.0), X, 0.0, aScale),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, aScale),
        trimmed_line(p(aScale, 0.0, 0.0), Y, 0.0, aScale),
    ]);
    (aProfiles, aGuides)
}

#[test]
fn GeomFill_Gordon__ScaledGeometry_LargeCoordinates() {
    // Large-scale geometry: coordinates in thousands.
    // Tests that scale-relative tolerances work correctly.
    let aScale = 1000.0;
    let (aProfiles, aGuides) = scaled_line_network(aScale);
    let aGordon = gordon_of(&aProfiles, &aGuides, 0.1);

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    let ((u1, u2), (v1, v2)) = (u_range(&aSurf), v_range(&aSurf));
    let aMidPt = aSurf.value(0.5 * (u1 + u2), 0.5 * (v1 + v2)).unwrap();

    // Center of a 1000x1000 bilinear patch should be near (500, 500, 0).
    assert_near(aMidPt.x(), 500.0, 1.0);
    assert_near(aMidPt.y(), 500.0, 1.0);
    assert_near(aMidPt.z(), 0.0, 1.0);
}

#[test]
fn GeomFill_Gordon__ScaledGeometry_SmallCoordinates() {
    // Micro-scale geometry: coordinates in millionths.
    let aScale = 1.0e-3;
    let (aProfiles, aGuides) = scaled_line_network(aScale);
    let aGordon = gordon_of(&aProfiles, &aGuides, aScale * 1.0e-4);

    assert!(aGordon.is_done());
    assert!(!aGordon.surface().unwrap().is_null());
}
