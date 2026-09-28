//=================================================================================================
// Tests for Geom2dGridEval_Curve (unified dispatcher)
//=================================================================================================

#[test]
fn Geom2dGridEval_CurveTest__LineDispatch() {
    let aGeomLine = Line(0.0, 0.0, 1.0, 0.0);
    let anAdaptor = Adaptor(&aGeomLine);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Line);

    let aParams = CreateUniformParams(0.0, 10.0, 11);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 11);

    for i in 1..=11 {
        let aExpected: Point2 = aGeomLine.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CurveTest__CircleDispatch() {
    let aGeomCircle = CircleOfRadius2();
    let anAdaptor = Adaptor(&aGeomCircle);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Circle);

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 17);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=17 {
        let aExpected: Point2 = aGeomCircle.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CurveTest__EllipseDispatch() {
    let anEllipse = Handle::new(Geom2dEllipse::new(OriginAxis(), 3.0, 2.0).unwrap());
    let anAdaptor = Adaptor(&anEllipse);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Ellipse);

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 13);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=13 {
        let aExpected: Point2 = anEllipse.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CurveTest__HyperbolaDispatch() {
    let aHypr = Handle::new(Geom2dHyperbola::new(OriginAxis(), 3.0, 2.0).unwrap());
    let anAdaptor = Adaptor(&aHypr);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Hyperbola);

    let aParams = CreateUniformParams(-2.0, 2.0, 11);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=11 {
        let aExpected: Point2 = aHypr.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CurveTest__ParabolaDispatch() {
    let aParab = Handle::new(Geom2dParabola::new(OriginAxis(), 1.0).unwrap());
    let anAdaptor = Adaptor(&aParab);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::Parabola);

    let aParams = CreateUniformParams(-2.0, 2.0, 11);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=11 {
        let aExpected: Point2 = aParab.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CurveTest__BSplineDispatch() {
    let aCurve = CreateSimpleBSpline2d();
    let anAdaptor = Adaptor(&aCurve);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::BSplineCurve);

    let aParams = CreateUniformParams(0.0, 1.0, 21);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=21 {
        let aExpected: Point2 = aCurve.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CurveTest__BezierCurveDispatch() {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 2.0);
    aPoles[3] = Point2::new(3.0, 2.0);
    aPoles[4] = Point2::new(4.0, 0.0);
    let aBezier = Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap());
    let anAdaptor = Adaptor(&aBezier);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::BezierCurve);

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=11 {
        let aExpected: Point2 = aBezier.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CurveTest__OffsetCurveDispatch() {
    let aLine: Handle<dyn Geom2dCurve> = Line(0.0, 0.0, 1.0, 0.0).cast();
    let anOffset = Handle::new(Geom2dOffsetCurve::new(&aLine, 1.0, false).unwrap());

    let anEval = Geom2dGridEvalCurve::from_curve(&anOffset.cast());
    assert_eq!(anEval.get_type(), CurveType::OffsetCurve);

    let aParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=6 {
        let aExpected: Point2 = anOffset.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CurveTest__DirectHandleInit() {
    let aGeomLine = Line(0.0, 0.0, 1.0, 0.0);

    let anEval = Geom2dGridEvalCurve::from_curve(&aGeomLine.cast());
    assert_eq!(anEval.get_type(), CurveType::Line);

    let aParams = CreateUniformParams(0.0, 10.0, 11);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 11);
    EXPECT_NEAR(aGrid[1].x(), 0.0, THE_TOLERANCE);
}

#[test]
fn Geom2dGridEval_CurveTest__EmptyParams() {
    let aGeomLine = Line(0.0, 0.0, 1.0, 0.0);
    let anAdaptor = Adaptor(&aGeomLine);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();

    let aEmptyParams: Array1<'static, f64> = Array1::default();
    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aEmptyParams).unwrap();
    assert!(aGrid.is_empty());
}

#[test]
fn Geom2dGridEval_CurveTest__UnifiedDerivativeD1() {
    let aGeomCircle = CircleOfRadius2();
    let anAdaptor = Adaptor(&aGeomCircle);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<Geom2dGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1) = aGeomCircle.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_CurveTest__UnifiedDerivativeD2() {
    let aCurve = CreateSimpleBSpline2d();
    let anAdaptor = Adaptor(&aCurve);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();

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
fn Geom2dGridEval_CurveTest__UnifiedDerivativeD3() {
    let aCurve = CreateSimpleBSpline2d();
    let anAdaptor = Adaptor(&aCurve);

    let anEval = Geom2dGridEvalCurve::from_adaptor(&anAdaptor).unwrap();

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
fn Geom2dGridEval_CurveTest__OffsetCurveDerivativeD3() {
    let aCircle: Handle<dyn Geom2dCurve> = CircleOfRadius2().cast();
    let anOffset = Handle::new(Geom2dOffsetCurve::new(&aCircle, 0.5, false).unwrap());

    let anEval = Geom2dGridEvalCurve::from_curve(&anOffset.cast());
    assert_eq!(anEval.get_type(), CurveType::OffsetCurve);

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<Geom2dGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1, aD2, aD3) = anOffset.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}
