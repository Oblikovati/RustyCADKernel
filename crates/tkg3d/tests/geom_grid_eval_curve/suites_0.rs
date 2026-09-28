// The comparison loops of the suite, over the first `theNb` grid records: points
// against a reference `Value`, and derivative records against reference `D1`..`D3`.
fn CheckPoints(theGrid: &Array1<'_, Point3>, theNb: i32, theExpected: impl Fn(i32) -> Point3) {
    for i in 1..=theNb {
        EXPECT_NEAR(theGrid[i].distance(&theExpected(i)), 0.0, THE_TOLERANCE);
    }
}

fn CheckD1(
    theGrid: &Array1<'_, GeomGridEvalCurveD1>,
    theNb: i32,
    theExpected: impl Fn(i32) -> (Point3, Vector3),
) {
    for i in 1..=theNb {
        let (aPnt, aD1) = theExpected(i);
        EXPECT_NEAR(theGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((theGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

fn CheckD2(
    theGrid: &Array1<'_, GeomGridEvalCurveD2>,
    theNb: i32,
    theExpected: impl Fn(i32) -> (Point3, Vector3, Vector3),
) {
    for i in 1..=theNb {
        let (aPnt, aD1, aD2) = theExpected(i);
        EXPECT_NEAR(theGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((theGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((theGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

fn CheckD3(
    theGrid: &Array1<'_, GeomGridEvalCurveD3>,
    theNb: i32,
    theExpected: impl Fn(i32) -> (Point3, Vector3, Vector3, Vector3),
) {
    for i in 1..=theNb {
        let (aPnt, aD1, aD2, aD3) = theExpected(i);
        EXPECT_NEAR(theGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((theGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((theGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((theGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}

//=================================================================================================
// Tests for GeomGridEval_Line
//=================================================================================================

#[test]
fn GeomGridEval_LineTest__BasicEvaluation() {
    // Create a Geom_Line along X axis starting at origin
    let aGeomLine = Line(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);

    let anEval = GeomGridEvalLine::new(aGeomLine);
    assert!(!anEval.geometry().is_null());

    let aParams = CreateUniformParams(0.0, 10.0, 11);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams);
    assert_eq!(aGrid.len(), 11);

    // Verify points
    for i in 1..=11 {
        let t: f64 = aParams[i];
        let aExpected = Point3::new(t, 0.0, 0.0);
        EXPECT_NEAR(aGrid[i].x(), aExpected.x(), THE_TOLERANCE);
        EXPECT_NEAR(aGrid[i].y(), aExpected.y(), THE_TOLERANCE);
        EXPECT_NEAR(aGrid[i].z(), aExpected.z(), THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_LineTest__NonOriginLine() {
    // Create a line at (1, 2, 3) along direction (1, 1, 1)/sqrt(3)
    let aGeomLine = Line(1.0, 2.0, 3.0, 1.0, 1.0, 1.0);

    let anEval = GeomGridEvalLine::new(aGeomLine);

    let aParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams);

    // Verify first and last point
    EXPECT_NEAR(
        aGrid[1].distance(&Point3::new(1.0, 2.0, 3.0)),
        0.0,
        THE_TOLERANCE,
    );

    // Line direction is normalized, so t=5 moves 5 units along normalized direction
    let d: f64 = 5.0 / 3.0_f64.sqrt();
    let aLast = Point3::new(1.0 + d, 2.0 + d, 3.0 + d);
    EXPECT_NEAR(aGrid[6].distance(&aLast), 0.0, THE_TOLERANCE);
}

//=================================================================================================
// Tests for GeomGridEval_Circle
//=================================================================================================

#[test]
fn GeomGridEval_CircleTest__BasicEvaluation() {
    // Circle in XY plane, radius 2, center at origin
    let aGeomCircle = CircleOfRadius2();

    let anEval = GeomGridEvalCircle::new(aGeomCircle);
    assert!(!anEval.geometry().is_null());

    // Test at 0, PI/2, PI, 3PI/2, 2PI
    let aParams = QuarterTurnParams();

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // u=0: (2, 0, 0)
    EXPECT_NEAR(aGrid[1].x(), 2.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[1].y(), 0.0, THE_TOLERANCE);

    // u=PI/2: (0, 2, 0)
    EXPECT_NEAR(aGrid[2].x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[2].y(), 2.0, THE_TOLERANCE);

    // u=PI: (-2, 0, 0)
    EXPECT_NEAR(aGrid[3].x(), -2.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[3].y(), 0.0, THE_TOLERANCE);

    // u=3PI/2: (0, -2, 0)
    EXPECT_NEAR(aGrid[4].x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[4].y(), -2.0, THE_TOLERANCE);

    // u=2PI: back to (2, 0, 0)
    EXPECT_NEAR(aGrid[5].x(), 2.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[5].y(), 0.0, THE_TOLERANCE);
}

#[test]
fn GeomGridEval_CircleTest__NonStandardCircle() {
    // Circle in YZ plane, radius 3, center at (1, 0, 0)
    let aGeomCircle = Handle::new(GeomCircle::new(Ax2(1.0, 0.0, 0.0, 1.0, 0.0, 0.0), 3.0).unwrap());

    let anEval = GeomGridEvalCircle::new(aGeomCircle);

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // All points should be at distance 3 from center (1, 0, 0)
    let aCenter = Point3::new(1.0, 0.0, 0.0);
    for i in 1..=aGrid.len() as i32 {
        EXPECT_NEAR(aGrid[i].distance(&aCenter), 3.0, THE_TOLERANCE);
        EXPECT_NEAR(aGrid[i].x(), 1.0, THE_TOLERANCE); // All points have X=1
    }
}

//=================================================================================================
// Tests for GeomGridEval_BSplineCurve
//=================================================================================================

#[test]
fn GeomGridEval_BSplineCurveTest__BasicEvaluation() {
    let aCurve = CreateSimpleBSpline();

    let anEval = GeomGridEvalBSplineCurve::new(aCurve.clone());
    assert!(!anEval.geometry().is_null());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 11);

    // Verify against direct evaluation
    CheckPoints(&aGrid, 11, |i| aCurve.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_BSplineCurveTest__EndpointsMatch() {
    let aCurve = CreateSimpleBSpline();

    let anEval = GeomGridEvalBSplineCurve::new(aCurve);

    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aParams[1] = 0.0;
    aParams[2] = 1.0;

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // First point should match first pole
    EXPECT_NEAR(
        aGrid[1].distance(&Point3::new(0.0, 0.0, 0.0)),
        0.0,
        THE_TOLERANCE,
    );

    // Last point should match last pole
    EXPECT_NEAR(
        aGrid[2].distance(&Point3::new(4.0, 0.0, 0.0)),
        0.0,
        THE_TOLERANCE,
    );
}

//=================================================================================================
// Tests for GeomGridEval_OtherCurve (fallback)
//=================================================================================================

#[test]
fn GeomGridEval_OtherCurveTest__EllipseFallback() {
    // Ellipse is not directly supported, so it should use OtherCurve fallback
    let anEllipse =
        Handle::new(GeomEllipse::new(Ax2(0.0, 0.0, 0.0, 0.0, 0.0, 1.0), 3.0, 2.0).unwrap());
    let anAdaptor = Adaptor(&anEllipse);

    let anEval = GeomGridEvalOtherCurve::new(&anAdaptor);

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, 9, |i| anEllipse.value(aParams[i]).unwrap());
}

//=================================================================================================
// Tests for GeomGridEval_Curve (unified dispatcher)
//=================================================================================================

#[test]
fn GeomGridEval_CurveTest__LineDispatch() {
    let aGeomLine = Line(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let anAdaptor = Adaptor(&aGeomLine);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Line);

    let aParams = CreateUniformParams(0.0, 10.0, 11);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 11);

    // Verify against direct evaluation
    CheckPoints(&aGrid, 11, |i| aGeomLine.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__CircleDispatch() {
    let aGeomCircle = CircleOfRadius2();
    let anAdaptor = Adaptor(&aGeomCircle);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Circle);

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 17);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, 17, |i| aGeomCircle.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__BSplineDispatch() {
    let aCurve = CreateSimpleBSpline();
    let anAdaptor = Adaptor(&aCurve);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::BSplineCurve);

    let aParams = CreateUniformParams(0.0, 1.0, 21);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, 21, |i| aCurve.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__EllipseDispatch() {
    let anEllipse =
        Handle::new(GeomEllipse::new(Ax2(0.0, 0.0, 0.0, 0.0, 0.0, 1.0), 3.0, 2.0).unwrap());
    let anAdaptor = Adaptor(&anEllipse);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Ellipse);

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 13);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, 13, |i| anEllipse.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__HyperbolaDispatch() {
    let aHypr =
        Handle::new(GeomHyperbola::new(Ax2(0.0, 0.0, 0.0, 0.0, 0.0, 1.0), 3.0, 2.0).unwrap());
    let anAdaptor = Adaptor(&aHypr);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Hyperbola);

    let aParams = CreateUniformParams(-2.0, 2.0, 11);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, 11, |i| aHypr.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__ParabolaDispatch() {
    let aParab = Handle::new(GeomParabola::new(Ax2(0.0, 0.0, 0.0, 0.0, 0.0, 1.0), 1.0).unwrap());
    let anAdaptor = Adaptor(&aParab);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Parabola);

    let aParams = CreateUniformParams(-2.0, 2.0, 11);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, 11, |i| aParab.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__BezierCurveDispatch() {
    // Bezier curve is not optimized, should use fallback
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(3.0, 2.0, 0.0);
    aPoles[4] = Point3::new(4.0, 0.0, 0.0);
    let aBezier = Handle::new(GeomBezierCurve::new(&aPoles).unwrap());
    let anAdaptor = Adaptor(&aBezier);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::BezierCurve);

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, 11, |i| aBezier.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__OffsetCurveFallbackDispatch() {
    // Offset curve is not optimized, should use fallback
    let aLine = Line(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let anOffset = OffsetDZ(&aLine, 1.0);
    let anAdaptor = Adaptor(&anOffset);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::OffsetCurve);

    let aParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, 6, |i| anOffset.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__DirectHandleInit() {
    let aGeomLine = Line(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);

    let anEval = GeomGridEvalCurve::from_curve(&aGeomLine.cast());
    assert_eq!(anEval.get_type(), CurveType::Line);

    let aParams = CreateUniformParams(0.0, 10.0, 11);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 11);
    EXPECT_NEAR(aGrid[1].x(), 0.0, THE_TOLERANCE);
}

#[test]
fn GeomGridEval_CurveTest__EmptyParams() {
    let aGeomLine = Line(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let anAdaptor = Adaptor(&aGeomLine);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();

    // EvaluateGrid with empty params should return empty
    let aEmptyParams: Array1<'static, f64> = Array1::default();
    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aEmptyParams).unwrap();
    assert!(aGrid.is_empty());
}
