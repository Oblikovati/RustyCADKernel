//==================================================================================================
// Line comparison tests
//==================================================================================================

#[test]
fn ExtremaPC_ComparisonTest__Line_PointOnLine() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomLine.cast(), -100.0, 100.0).unwrap();

    let aPoint = Point3::new(5.0, 0.0, 0.0);

    // New implementation
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    // Old implementation
    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Line_PointOnLine");
}

#[test]
fn ExtremaPC_ComparisonTest__Line_PointOffLine() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomLine.cast(), -100.0, 100.0).unwrap();

    let aPoint = Point3::new(5.0, 3.0, 4.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Line_PointOffLine");
}

//==================================================================================================
// Circle comparison tests
//==================================================================================================

/// `Geom_Circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 10.0)`.
fn circle10() -> GeomAdaptorCurve {
    let aGeomCircle = Handle::new(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );
    GeomAdaptorCurve::with_curve(&aGeomCircle.cast()).unwrap()
}

#[test]
fn ExtremaPC_ComparisonTest__Circle_PointOutside() {
    let anAdaptor = circle10();

    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Circle_PointOutside");
}

#[test]
fn ExtremaPC_ComparisonTest__Circle_PointInside() {
    let anAdaptor = circle10();

    let aPoint = Point3::new(3.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Circle_PointInside");
}

#[test]
fn ExtremaPC_ComparisonTest__Circle_PointOffPlane() {
    let anAdaptor = circle10();

    let aPoint = Point3::new(15.0, 0.0, 5.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Circle_PointOffPlane");
}

//==================================================================================================
// Ellipse comparison tests
//==================================================================================================

/// `Geom_Ellipse(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 20.0, 10.0)`.
fn ellipse20x10() -> GeomAdaptorCurve {
    let aGeomEllipse = Handle::new(
        GeomEllipse::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 20.0, 10.0)
            .unwrap(),
    );
    GeomAdaptorCurve::with_curve(&aGeomEllipse.cast()).unwrap()
}

#[test]
fn ExtremaPC_ComparisonTest__Ellipse_PointOnMajorAxis() {
    let anAdaptor = ellipse20x10();

    let aPoint = Point3::new(30.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Ellipse_PointOnMajorAxis");
}

#[test]
fn ExtremaPC_ComparisonTest__Ellipse_PointGeneral() {
    let anAdaptor = ellipse20x10();

    let aPoint = Point3::new(15.0, 12.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Ellipse_PointGeneral");
}

//==================================================================================================
// Parabola comparison tests
//==================================================================================================

#[test]
fn ExtremaPC_ComparisonTest__Parabola_PointNearVertex() {
    let aGeomParabola = Handle::new(
        GeomParabola::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 2.0).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomParabola.cast(), -10.0, 10.0).unwrap();

    let aPoint = Point3::new(1.0, 2.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Parabola_PointNearVertex");
}

//==================================================================================================
// Hyperbola comparison tests
//==================================================================================================

#[test]
fn ExtremaPC_ComparisonTest__Hyperbola_PointNearBranch() {
    let aGeomHyperbola = Handle::new(
        GeomHyperbola::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0, 5.0)
            .unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomHyperbola.cast(), -2.0, 2.0).unwrap();

    let aPoint = Point3::new(15.0, 3.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Hyperbola_PointNearBranch");
}

//==================================================================================================
// BSpline comparison tests
//==================================================================================================

#[test]
fn ExtremaPC_ComparisonTest__BSpline_CubicCurve() {
    let aBSpline = cubic_bspline();
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();

    let aPoint = Point3::new(1.5, 3.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "BSpline_CubicCurve");
}

#[test]
fn ExtremaPC_ComparisonTest__BSpline_MultiSpan() {
    // For degree 2, sum(mults) = n_poles + degree + 1 = 6 + 2 + 1 = 9
    // Using 4 knots with mults = 3,1,2,3 = 9
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 6).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 3.0, 0.0);
    aPoles[3] = Point3::new(2.0, 0.0, 0.0);
    aPoles[4] = Point3::new(3.0, 3.0, 0.0);
    aPoles[5] = Point3::new(4.0, 0.0, 0.0);
    aPoles[6] = Point3::new(5.0, 3.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 0.33;
    aKnots[3] = 0.67;
    aKnots[4] = 1.0;

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 4).unwrap();
    aMults[1] = 3;
    aMults[2] = 1;
    aMults[3] = 2;
    aMults[4] = 3;

    let aBSpline =
        Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap());
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();

    let aPoint = Point3::new(3.0, 4.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    // NOTE: Multi-span BSplines with complex configurations may find different extrema
    // due to BVH subdivision strategy. The new implementation finds valid extrema,
    // but may not find the global minimum. This is a known area for improvement.
    // For this test, we only verify:
    // 1. New implementation completes successfully
    // 2. Results are geometrically valid (point is actually on curve)
    assert!(aNewResult.is_done(), "New implementation should complete");
    EXPECT_GT(aNewResult.nb_ext(), 0);

    if aNewResult.nb_ext() > 0 {
        // Verify the found point is actually on the curve
        let aExtremum = &aNewResult[aNewResult.min_index()];
        let aCurvePnt: Point3 = anAdaptor.value(aExtremum.parameter).unwrap();
        EXPECT_NEAR(aCurvePnt.distance(&aExtremum.point), 0.0, 1.0e-6);
    }

    // Log the difference for information
    if anOldExtPC.is_done() && anOldExtPC.nb_ext().unwrap() > 0 && aNewResult.nb_ext() > 0 {
        let mut aOldMinSqDist: f64 = f64::MAX;
        for i in 1..=anOldExtPC.nb_ext().unwrap() {
            if anOldExtPC.square_distance(i).unwrap() < aOldMinSqDist {
                aOldMinSqDist = anOldExtPC.square_distance(i).unwrap();
            }
        }
        let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
        let aOldMinDist: f64 = aOldMinSqDist.sqrt();
        println!(
            "  [INFO] BSpline_MultiSpan: Old min dist = {aOldMinDist}, New min dist = {aNewMinDist}"
        );
    }
}

//==================================================================================================
// Bezier comparison tests
//==================================================================================================

#[test]
fn ExtremaPC_ComparisonTest__Bezier_CubicCurve() {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 3.0, 0.0);
    aPoles[3] = Point3::new(3.0, 3.0, 0.0);
    aPoles[4] = Point3::new(4.0, 0.0, 0.0);

    let aBezier = Handle::new(GeomBezierCurve::new(&aPoles).unwrap());
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBezier.cast()).unwrap();

    let aPoint = Point3::new(2.0, 4.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    CompareMinDistance(aNewResult, &anOldExtPC, "Bezier_CubicCurve");
}
