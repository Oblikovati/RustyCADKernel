//=================================================================================================
// Tests for Geom2dGridEval_Line
//=================================================================================================

#[test]
fn Geom2dGridEval_LineTest__BasicEvaluation() {
    let aGeomLine = Line(0.0, 0.0, 1.0, 0.0);

    let anEval = Geom2dGridEvalLine::new(aGeomLine);
    assert!(!anEval.geometry().is_null());

    let aParams = CreateUniformParams(0.0, 10.0, 11);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams);
    assert_eq!(aGrid.len(), 11);

    for i in 1..=11 {
        let t: f64 = aParams[i];
        EXPECT_NEAR(aGrid[i].x(), t, THE_TOLERANCE);
        EXPECT_NEAR(aGrid[i].y(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_LineTest__NonOriginLine() {
    let aGeomLine = Line(1.0, 2.0, 1.0, 1.0);

    let anEval = Geom2dGridEvalLine::new(aGeomLine);

    let aParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams);

    EXPECT_NEAR(aGrid[1].distance(Point2::new(1.0, 2.0)), 0.0, THE_TOLERANCE);

    let d: f64 = 5.0 / 2.0_f64.sqrt();
    EXPECT_NEAR(aGrid[6].distance(Point2::new(1.0 + d, 2.0 + d)), 0.0, THE_TOLERANCE);
}

#[test]
fn Geom2dGridEval_LineTest__DerivativeD1() {
    let aGeomLine = Line(0.0, 0.0, 1.0, 2.0);
    let anEval = Geom2dGridEvalLine::new(aGeomLine);

    let aParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array1<Geom2dGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams);
    assert_eq!(aGrid.len(), 6);

    let aDir = Dir(1.0, 2.0);
    for i in 1..=6 {
        EXPECT_NEAR(aGrid[i].d1.x(), aDir.x(), THE_TOLERANCE);
        EXPECT_NEAR(aGrid[i].d1.y(), aDir.y(), THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_LineTest__DerivativeD2D3() {
    let aGeomLine = Line(0.0, 0.0, 1.0, 0.0);
    let anEval = Geom2dGridEvalLine::new(aGeomLine);

    let aParams = CreateUniformParams(0.0, 5.0, 6);

    let aGridD2: Array1<Geom2dGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams);
    let aGridD3: Array1<Geom2dGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams);

    for i in 1..=6 {
        EXPECT_NEAR(aGridD2[i].d2.magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR(aGridD3[i].d2.magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR(aGridD3[i].d3.magnitude(), 0.0, THE_TOLERANCE);
    }
}

//=================================================================================================
// Tests for Geom2dGridEval_Circle
//=================================================================================================

#[test]
fn Geom2dGridEval_CircleTest__BasicEvaluation() {
    let aGeomCircle = CircleOfRadius2();

    let anEval = Geom2dGridEvalCircle::new(aGeomCircle);
    assert!(!anEval.geometry().is_null());

    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 5).unwrap();
    aParams[1] = 0.0;
    aParams[2] = PI / 2.0;
    aParams[3] = PI;
    aParams[4] = 3.0 * PI / 2.0;
    aParams[5] = 2.0 * PI;

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    EXPECT_NEAR(aGrid[1].x(), 2.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[1].y(), 0.0, THE_TOLERANCE);

    EXPECT_NEAR(aGrid[2].x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[2].y(), 2.0, THE_TOLERANCE);

    EXPECT_NEAR(aGrid[3].x(), -2.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[3].y(), 0.0, THE_TOLERANCE);

    EXPECT_NEAR(aGrid[4].x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[4].y(), -2.0, THE_TOLERANCE);

    EXPECT_NEAR(aGrid[5].x(), 2.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[5].y(), 0.0, THE_TOLERANCE);
}

#[test]
fn Geom2dGridEval_CircleTest__DerivativeD1() {
    let aGeomCircle = CircleOfRadius2();
    let anEval = Geom2dGridEvalCircle::new(aGeomCircle.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<Geom2dGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1) = aGeomCircle.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CircleTest__DerivativeD2() {
    let aGeomCircle = CircleOfRadius2();
    let anEval = Geom2dGridEvalCircle::new(aGeomCircle.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<Geom2dGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1, aD2) = aGeomCircle.d2(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CircleTest__DerivativeD3() {
    let aGeomCircle = CircleOfRadius2();
    let anEval = Geom2dGridEvalCircle::new(aGeomCircle.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<Geom2dGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1, aD2, aD3) = aGeomCircle.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}

//=================================================================================================
// Tests for Geom2dGridEval_BSplineCurve
//=================================================================================================

#[test]
fn Geom2dGridEval_BSplineCurveTest__BasicEvaluation() {
    let aCurve = CreateSimpleBSpline2d();

    let anEval = Geom2dGridEvalBSplineCurve::new(aCurve.clone());
    assert!(!anEval.geometry().is_null());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 11);

    for i in 1..=11 {
        let aExpected: Point2 = aCurve.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BSplineCurveTest__EndpointsMatch() {
    let aCurve = CreateSimpleBSpline2d();

    let anEval = Geom2dGridEvalBSplineCurve::new(aCurve);

    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aParams[1] = 0.0;
    aParams[2] = 1.0;

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    EXPECT_NEAR(aGrid[1].distance(Point2::new(0.0, 0.0)), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[2].distance(Point2::new(4.0, 0.0)), 0.0, THE_TOLERANCE);
}

#[test]
fn Geom2dGridEval_BSplineCurveTest__DerivativeD1() {
    let aCurve = CreateSimpleBSpline2d();
    let anEval = Geom2dGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Geom2dGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=11 {
        let (aPnt, aD1) = aCurve.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BSplineCurveTest__DerivativeD2() {
    let aCurve = CreateSimpleBSpline2d();
    let anEval = Geom2dGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Geom2dGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    for i in 1..=11 {
        let (aPnt, aD1, aD2) = aCurve.d2(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BSplineCurveTest__DerivativeD3() {
    let aCurve = CreateSimpleBSpline2d();
    let anEval = Geom2dGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Geom2dGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=11 {
        let (aPnt, aD1, aD2, aD3) = aCurve.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BSplineCurveTest__DerivativeDN_BeyondDegree() {
    let aCurve = CreateSimpleBSpline2d();
    let anEval = Geom2dGridEvalBSplineCurve::new(aCurve);

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Vector2> = anEval.evaluate_grid_dn(&aParams, 4).unwrap();

    for i in 1..=11 {
        EXPECT_NEAR(aGrid[i].magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BSplineCurveTest__MultiSpanBSpline() {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 6).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 2.0);
    aPoles[3] = Point2::new(2.0, 2.0);
    aPoles[4] = Point2::new(3.0, 0.0);
    aPoles[5] = Point2::new(4.0, -1.0);
    aPoles[6] = Point2::new(5.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 3).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 0.5;
    aKnots[3] = 1.0;
    aMults[1] = 4;
    aMults[2] = 2;
    aMults[3] = 4;

    let aCurve =
        Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap());

    let anEval = Geom2dGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 31);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=31 {
        let aExpected: Point2 = aCurve.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}
