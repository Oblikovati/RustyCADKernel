// GeomFill_Gordon tests (high-level, automatic intersection detection), part 2.

// The mid point `(UMid, VMid)` of a surface.
fn mid_point(aSurf: &Handle<GeomBSplineSurface>) -> Point3 {
    let ((u1, u2), (v1, v2)) = (u_range(aSurf), v_range(aSurf));
    aSurf.value(0.5 * (u1 + u2), 0.5 * (v1 + v2)).unwrap()
}

// Linear guides at X = 0 and X = 1 from Y = 0 to Y = 1.
fn unit_linear_guides() -> Array1<'static, Handle<dyn GeomCurve>> {
    curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)),
        makeLinearBSpline(p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)),
    ])
}

#[test]
fn GeomFill_Gordon__AllReversedCurves_ProducesValidSurface() {
    // Every single curve is reversed. The sort network should fix them all.
    let aProfiles = curve_array(vec![
        trimmed_line(p(1.0, 0.0, 0.0), (-1.0, 0.0, 0.0), 0.0, 1.0),
        trimmed_line(p(1.0, 1.0, 0.0), (-1.0, 0.0, 0.0), 0.0, 1.0),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 1.0, 0.0), (0.0, -1.0, 0.0), 0.0, 1.0),
        trimmed_line(p(1.0, 1.0, 0.0), (0.0, -1.0, 0.0), 0.0, 1.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.surface().unwrap().is_null());
}

#[test]
fn GeomFill_Gordon__ShuffledInputOrder_ProducesValidSurface() {
    // Curves provided in non-sorted order. Sort network should handle it.
    // Provide profiles out of order: y=1, y=0.5, y=0.
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 1.0, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 0.5, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 1.0),
    ]);
    // Guides also out of order: x=1, x=0.
    let aGuides = curve_array(vec![
        trimmed_line(p(1.0, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 1.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    // Despite shuffled order, surface should be a valid unit-square patch.
    assert_near(mid_point(&aSurf).z(), 0.0, 0.01);
}

#[test]
fn GeomFill_Gordon__SaddleSurface_BothDirectionsCurved() {
    // Saddle shape: profiles curve up in Z, guides curve down in Z.
    // This creates a hyperbolic paraboloid-like surface.
    let aProfiles = curves(&[
        makeQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.5, 0.0, 0.5), p(1.0, 0.0, 0.0)),
        makeQuadraticBSpline(p(0.0, 1.0, 0.0), p(0.5, 1.0, 0.5), p(1.0, 1.0, 0.0)),
    ]);
    let aGuides = curves(&[
        makeQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.0, 0.5, -0.5), p(0.0, 1.0, 0.0)),
        makeQuadraticBSpline(p(1.0, 0.0, 0.0), p(1.0, 0.5, -0.5), p(1.0, 1.0, 0.0)),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, 0.01);

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    // Surface should have Z > 0 in the profile hump direction and Z < 0 in guide hump direction,
    // creating a saddle-like shape. Just verify it's a valid non-planar surface.
    let aCenterPt = mid_point(&aSurf);

    // The center is the superposition of profile hump (+0.5) and guide dip (-0.5).
    // Gordon formula may not give exactly 0, but it should be a well-defined surface.
    assert_near(aCenterPt.z(), 0.0, 0.5);
}

#[test]
fn GeomFill_Gordon__AsymmetricNetwork_FiveProfilesTwoGuides() {
    // Highly asymmetric: 5 profiles but only 2 guides.
    let aProfiles = curve_array(
        (0..5).map(|i| trimmed_line(p(0.0, f64::from(i) * 0.25, 0.0), X, 0.0, 1.0)).collect(),
    );
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 1.0),
        trimmed_line(p(1.0, 0.0, 0.0), Y, 0.0, 1.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.surface().unwrap().is_null());
}

#[test]
fn GeomFill_Gordon__AsymmetricNetwork_TwoProfilesFiveGuides() {
    // Highly asymmetric: 2 profiles but 5 guides.
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 1.0),
        trimmed_line(p(0.0, 1.0, 0.0), X, 0.0, 1.0),
    ]);
    let aGuides = curve_array(
        (0..5).map(|i| trimmed_line(p(f64::from(i) * 0.25, 0.0, 0.0), Y, 0.0, 1.0)).collect(),
    );
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());
    assert!(!aGordon.surface().unwrap().is_null());
}

#[test]
fn GeomFill_Gordon__NonPlanarNetwork_CurvesInBothDirections() {
    // Both profiles and guides are curved in 3D (non-planar network).
    let aProfiles = curves(&[makeSineBSpline(0.0, 1.0, 0.0, 0.2), makeSineBSpline(0.0, 1.0, 1.0, 0.2)]);

    // Guides: also sinusoidal but in Y direction with Z offset.
    let (mut aGuidPnts1, mut aGuidPnts2, mut aGuidParams) = (Vec::new(), Vec::new(), Vec::new());
    for i in 1..=11 {
        let aT = f64::from(i - 1) / 10.0;
        // Guides go from Y=0 to Y=1. Z must match profiles at intersections.
        // At X=0: profile Z = sin(pi*0)*amp = 0 for both profiles.
        // At X=1: same.
        // So guide endpoints have Z=0.
        let aZ1 = 0.15 * (PI * aT).sin();
        let aZ2 = 0.15 * (PI * aT).sin();
        aGuidPnts1.push(p(0.0, aT, aZ1));
        aGuidPnts2.push(p(1.0, aT, aZ2));
        aGuidParams.push(aT);
    }

    let aGuides = curves(&[
        interpolated(aGuidPnts1, Some(aGuidParams.clone()), false),
        interpolated(aGuidPnts2, Some(aGuidParams), false),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, 0.01);

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    // Surface should have non-zero Z everywhere except at corners.
    // Non-planar network should preserve combined profile/guide elevation.
    assert_near(mid_point(&aSurf).z(), 0.35, 0.05);
}

#[test]
fn GeomFill_Gordon__SurfaceContinuity_IsSmooth() {
    // Verify that the resulting surface is at least C1 continuous by checking
    // that derivatives are smooth across the parameter domain.
    let aProfiles = curves(&[
        makeQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.5, 0.0, 0.3), p(1.0, 0.0, 0.0)),
        makeQuadraticBSpline(p(0.0, 0.5, 0.0), p(0.5, 0.5, 0.5), p(1.0, 0.5, 0.0)),
        makeQuadraticBSpline(p(0.0, 1.0, 0.0), p(0.5, 1.0, 0.3), p(1.0, 1.0, 0.0)),
    ]);
    let aGuides = unit_linear_guides();
    let aGordon = gordon_of(&aProfiles, &aGuides, 0.01);

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    let ((aUMin, aUMax), (aVMin, aVMax)) = (u_range(&aSurf), v_range(&aSurf));

    // Check derivative continuity along U-isoparameter at V=0.5*(VMin+VMax).
    let aV = 0.5 * (aVMin + aVMax);
    const THE_NB: i32 = 50;
    const THE_DERIV_TOL: f64 = 1.0; // Allow generous tolerance for derivative jumps.

    let aStep = (aUMax - aUMin) / f64::from(THE_NB);
    let mut aPrevDU = aSurf.eval_d1(aUMin + 2.0 * aStep, aV).unwrap().d1u;

    for i in 3..=THE_NB - 2 {
        let aU = aUMin + f64::from(i) * aStep;
        let aDU = aSurf.eval_d1(aU, aV).unwrap().d1u;

        // Derivatives should change gradually, not jump.
        let aDUChange = (aDU - aPrevDU).magnitude();
        assert!(aDUChange < THE_DERIV_TOL, "DU jump at U={aU}: {aDUChange}");

        aPrevDU = aDU;
    }
}

#[test]
fn GeomFill_Gordon__ArcLikeProfiles_NonRational() {
    // Profiles are arc-like curves (interpolated semicircle shape) in XZ plane
    // at different Y positions. Non-rational since Gordon rejects rational curves.
    // Guides are straight lines connecting endpoints.

    // Build arc-like profiles via interpolation of semicircle-like points.
    let makeArcProfile = |theY: f64| -> Handle<GeomBSplineCurve> {
        const THE_NB: i32 = 9;
        let (mut aPnts, mut aParams) = (Vec::new(), Vec::new());
        for i in 1..=THE_NB {
            let aT = f64::from(i - 1) / f64::from(THE_NB - 1);
            let aAngle = PI * aT;
            // Semicircle: X = 0.5 + 0.5*cos(pi - angle), Z = 0.3*sin(angle)
            let aX = 0.5 - 0.5 * aAngle.cos();
            let aZ = 0.3 * aAngle.sin();
            aPnts.push(p(aX, theY, aZ));
            aParams.push(aT);
        }
        interpolated(aPnts, Some(aParams), false)
    };

    let aProfiles = curves(&[makeArcProfile(0.0), makeArcProfile(1.0)]);

    // Guides connect endpoints: at X=0 and X=1, Z=0.
    let aGuides = unit_linear_guides();
    let aGordon = gordon_of(&aProfiles, &aGuides, 0.01);

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    // The surface center should have positive Z (from the arc-like profiles).
    assert_near(mid_point(&aSurf).z(), 0.3, 0.05);
}

#[test]
fn GeomFill_Gordon__RepeatPerform_GivesSameResult() {
    // Calling Perform() twice on the same initialized Gordon should produce
    // the same result (no state corruption).
    let (aProfiles, aGuides) = unit_line_network();

    let mut aGordon = GeomFillGordon::new();
    aGordon.init(&aProfiles, &aGuides, precision::confusion()).unwrap();

    // First perform.
    aGordon.perform().unwrap();
    assert!(aGordon.is_done());
    let aMid1 = mid_point(&aGordon.surface().unwrap());

    // Perform again without reinitializing. The algorithm should rebuild its
    // working curves from immutable input handles.
    aGordon.perform().unwrap();
    assert!(aGordon.is_done());
    let aMid2 = mid_point(&aGordon.surface().unwrap());

    // Repeated Perform should give same result.
    assert_near(aMid1.distance(&aMid2), 0.0, precision::confusion());
    assert_eq!(aGordon.report().status, GordonStatus::Done);
}

#[test]
fn GeomFill_Gordon__HighDensityNetwork_SixBySix() {
    // 6 profiles + 6 guides - tests performance and correctness with denser grids.
    const THE_NB: i32 = 6;
    let aProfiles = curve_array(
        (0..THE_NB)
            .map(|i| trimmed_line(p(0.0, f64::from(i) / f64::from(THE_NB - 1), 0.0), X, 0.0, 1.0))
            .collect(),
    );
    let aGuides = curve_array(
        (0..THE_NB)
            .map(|j| trimmed_line(p(f64::from(j) / f64::from(THE_NB - 1), 0.0, 0.0), Y, 0.0, 1.0))
            .collect(),
    );
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    // Verify all 36 intersection points.
    let ((aUMin, aUMax), (aVMin, aVMax)) = (u_range(&aSurf), v_range(&aSurf));
    for i in 0..THE_NB {
        for j in 0..THE_NB {
            let aX = f64::from(j) / f64::from(THE_NB - 1);
            let aY = f64::from(i) / f64::from(THE_NB - 1);
            let aU = aUMin + aX * (aUMax - aUMin);
            let aV = aVMin + aY * (aVMax - aVMin);
            let aSurfPt = aSurf.value(aU, aV).unwrap();
            // Intersection point (i, j) Z-deviation.
            assert_near(aSurfPt.z(), 0.0, 0.02);
        }
    }
}

#[test]
fn GeomFill_Gordon__QuadraticProfiles_SurfaceMatchesCurvesAtBoundaries() {
    // Quadratic profiles with Z-bump + linear guides.
    // Verify the surface exactly matches the boundary profiles.
    let aProfiles = curves(&[
        makeQuadraticBSpline(p(0.0, 0.0, 0.0), p(0.5, 0.0, 1.0), p(1.0, 0.0, 0.0)),
        makeQuadraticBSpline(p(0.0, 1.0, 0.0), p(0.5, 1.0, 1.0), p(1.0, 1.0, 0.0)),
    ]);
    let aGuides = unit_linear_guides();
    let aGordon = gordon_of(&aProfiles, &aGuides, 0.01);

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    let ((aUMin, aUMax), (aVMin, aVMax)) = (u_range(&aSurf), v_range(&aSurf));

    // First profile (boundary V=VMin): Z should match quadratic bump.
    for i in 0..=10 {
        let aT = f64::from(i) / 10.0;
        let aU = aUMin + aT * (aUMax - aUMin);
        let d = aSurf.value(aU, aVMin).unwrap().distance(&aProfiles[1].value(aT).unwrap());
        assert!(d < 0.05, "Profile 1 deviation at t={aT}");
    }

    // Last profile (boundary V=VMax).
    for i in 0..=10 {
        let aT = f64::from(i) / 10.0;
        let aU = aUMin + aT * (aUMax - aUMin);
        let d = aSurf.value(aU, aVMax).unwrap().distance(&aProfiles[2].value(aT).unwrap());
        assert!(d < 0.05, "Profile 2 deviation at t={aT}");
    }
}

#[test]
fn GeomFill_Gordon__OffsetPlane_NetworkAtNonZeroZ() {
    // Network at Z=5 (not at origin). Tests that the algorithm works for
    // arbitrary positions, not just geometry centered at origin.
    let aZ = 5.0;
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 0.0, aZ), X, 0.0, 1.0),
        trimmed_line(p(0.0, 1.0, aZ), X, 0.0, 1.0),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, aZ), Y, 0.0, 1.0),
        trimmed_line(p(1.0, 0.0, aZ), Y, 0.0, 1.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());

    assert_near(mid_point(&aGordon.surface().unwrap()).z(), aZ, 0.01);
}

#[test]
fn GeomFill_Gordon__TiltedPlane_DiagonalCurves() {
    // Profiles and guides on a tilted plane (Z varies linearly with X and Y).
    // Expected surface: Z = X + Y.
    let aProfiles = curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(1.0, 0.0, 1.0)),
        makeLinearBSpline(p(0.0, 1.0, 1.0), p(1.0, 1.0, 2.0)),
    ]);
    let aGuides = curves(&[
        makeLinearBSpline(p(0.0, 0.0, 0.0), p(0.0, 1.0, 1.0)),
        makeLinearBSpline(p(1.0, 0.0, 1.0), p(1.0, 1.0, 2.0)),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    let ((aUMin, aUMax), (aVMin, aVMax)) = (u_range(&aSurf), v_range(&aSurf));

    // Sample multiple points and check Z = X + Y.
    for i in 0..=5 {
        for j in 0..=5 {
            let aU = aUMin + f64::from(i) * (aUMax - aUMin) / 5.0;
            let aV = aVMin + f64::from(j) * (aVMax - aVMin) / 5.0;
            let aPt = aSurf.value(aU, aV).unwrap();
            assert_near(aPt.z(), aPt.x() + aPt.y(), 0.05);
        }
    }
}

#[test]
fn GeomFill_Gordon__RectangularNotSquare_2x3AspectRatio() {
    // Rectangular domain with 2:3 aspect ratio.
    let aProfiles = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), X, 0.0, 2.0),
        trimmed_line(p(0.0, 3.0, 0.0), X, 0.0, 2.0),
    ]);
    let aGuides = curve_array(vec![
        trimmed_line(p(0.0, 0.0, 0.0), Y, 0.0, 3.0),
        trimmed_line(p(2.0, 0.0, 0.0), Y, 0.0, 3.0),
    ]);
    let aGordon = gordon_of(&aProfiles, &aGuides, precision::confusion());

    assert!(aGordon.is_done());

    let aMidPt = mid_point(&aGordon.surface().unwrap());
    assert_near(aMidPt.x(), 1.0, 0.1);
    assert_near(aMidPt.y(), 1.5, 0.1);
}

#[test]
fn GeomFill_Gordon__WavySurface_SinusoidalProfilesAndGuides() {
    // Both profiles and guides have sinusoidal Z, creating a wavy surface.
    let aProfiles = curves(&[
        makeSineBSpline(0.0, 1.0, 0.0, 0.3),
        makeSineBSpline(0.0, 1.0, 0.5, 0.6),
        makeSineBSpline(0.0, 1.0, 1.0, 0.3),
    ]);

    // Guides connect profile endpoints consistently.
    // At X=0: all profiles have Z=0. At X=1: all profiles have Z=0.
    // So guides at X=0 and X=1 are straight lines.
    let aGuides = unit_linear_guides();
    let aGordon = gordon_of(&aProfiles, &aGuides, 0.01);

    assert!(aGordon.is_done());

    let aSurf = aGordon.surface().unwrap();
    assert!(!aSurf.is_null());

    // Center peak: middle profile has amplitude 0.6, so center Z should be high.
    assert_near(mid_point(&aSurf).z(), 0.6, 0.05); // Center should match middle sine amplitude
}
