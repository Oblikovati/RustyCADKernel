//==================================================================================================
// Offset curve comparison tests
//==================================================================================================

#[test]
fn ExtremaPC_ComparisonTest__OffsetCircle() {
    let aCircle = Handle::new(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    let aPoint = Point3::new(25.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "OffsetCircle");
}

//==================================================================================================
// Random point stress tests
//==================================================================================================

// The shared loop of the random tests: counts the points where the new minimum is
// worse than the old one beyond the tolerances.
fn random_fail_count(anAdaptor: &GeomAdaptorCurve, aPoints: &[Point3]) -> i32 {
    let mut aFailCount: i32 = 0;
    for &aPoint in aPoints {
        let mut anExtPC = ExtremaPC::Curve::from_adaptor(anAdaptor).unwrap();
        let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

        let anOldExtPC = old_ext_pc(aPoint, anAdaptor);

        if aNewResult.is_done()
            && anOldExtPC.is_done()
            && aNewResult.nb_ext() > 0
            && anOldExtPC.nb_ext().unwrap() > 0
        {
            let aNewMin: f64 = aNewResult.min_square_distance().sqrt();
            let mut aOldMin: f64 = f64::MAX;
            for j in 1..=anOldExtPC.nb_ext().unwrap() {
                aOldMin = aOldMin.min(anOldExtPC.square_distance(j).unwrap().sqrt());
            }

            // New should be at least as good
            if aNewMin > aOldMin * (1.0 + THE_TOL_REL) + THE_TOL {
                aFailCount += 1;
            }
        }
    }
    aFailCount
}

#[test]
fn ExtremaPC_ComparisonTest__Circle_RandomPoints() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&aGeomCircle.cast()).unwrap();

    // std::mt19937 aGen(12345), uniform_real_distribution(-30.0, 30.0): fixed seed for
    // reproducibility
    let aPoints = random_points("comparison_circle");
    let aNumTests: usize = 100;
    assert_eq!(aPoints.len(), aNumTests);

    let aFailCount: i32 = random_fail_count(&anAdaptor, &aPoints);

    assert_eq!(
        aFailCount, 0,
        "Circle_RandomPoints: {aFailCount} out of {aNumTests} tests had worse results"
    );
}

#[test]
fn ExtremaPC_ComparisonTest__BSpline_RandomPoints() {
    // For degree 2, sum(mults) = n_poles + degree + 1 = 5 + 2 + 1 = 8
    // Using 4 knots with mults = 3,1,1,3 = 8
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 5).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 1.0);
    aPoles[3] = Point3::new(2.0, 1.0, 2.0);
    aPoles[4] = Point3::new(3.0, 3.0, 1.0);
    aPoles[5] = Point3::new(4.0, 0.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 0.33;
    aKnots[3] = 0.67;
    aKnots[4] = 1.0;

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 4).unwrap();
    aMults[1] = 3;
    aMults[2] = 1;
    aMults[3] = 1;
    aMults[4] = 3;

    let aBSpline =
        Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap());
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();

    // std::mt19937 aGen(54321), uniform_real_distribution(-2.0, 7.0)
    let aPoints = random_points("comparison_bspline");
    let aNumTests: usize = 100;
    assert_eq!(aPoints.len(), aNumTests);

    let aFailCount: i32 = random_fail_count(&anAdaptor, &aPoints);

    assert_eq!(
        aFailCount, 0,
        "BSpline_RandomPoints: {aFailCount} out of {aNumTests} tests had worse results"
    );
}

//==================================================================================================
// Edge case comparison tests
//==================================================================================================

#[test]
fn ExtremaPC_ComparisonTest__PointOnCurve_Circle() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&aGeomCircle.cast()).unwrap();

    // Point exactly on the circle
    let aPoint = Point3::new(10.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "PointOnCurve_Circle");

    // Distance should be ~0
    EXPECT_LT(aNewResult.min_square_distance().sqrt(), THE_TOL);
}

#[test]
fn ExtremaPC_ComparisonTest__PointOnCurve_BSpline() {
    let aBSpline = cubic_bspline();
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();

    // Point on curve at u=0.5
    let aPoint: Point3 = GeomCurve::value(&*aBSpline, 0.5).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "PointOnCurve_BSpline");

    EXPECT_LT(aNewResult.min_square_distance().sqrt(), THE_TOL);
}
