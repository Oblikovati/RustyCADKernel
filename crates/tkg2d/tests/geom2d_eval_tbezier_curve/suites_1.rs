// Test Copy produces independent identical object
#[test]
fn Geom2dEval_TBezierCurveTest__Copy_Independent() {
    let aCurve = createSimpleCurve();
    let aCopy = aCurve.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyCurve: Handle<Geom2dEvalTBezierCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());
    assert_eq!(aCopyCurve.nb_poles(), 3);
    EXPECT_NEAR(aCopyCurve.alpha(), 1.0, precision::confusion());

    // Verify copy evaluates the same
    let aU: f64 = PI / 4.0;
    let aOrigPt: Point2 = aCurve.eval_d0(aU).unwrap();
    let aCopyPt: Point2 = aCopyCurve.eval_d0(aU).unwrap();
    EXPECT_NEAR(aOrigPt.distance(aCopyPt), 0.0, precision::confusion());
}

// Test Copy behavior with unsupported transform.
#[test]
fn Geom2dEval_TBezierCurveTest__Copy_Modification_Independent() {
    let aCurve = createSimpleCurve();
    let aCopy = aCurve.copy().unwrap();
    let aCopyCurve: Handle<Geom2dEvalTBezierCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());

    // Transform is unsupported on the copy as well.
    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(10.0, 10.0));
    EXPECT_THROW(aCopyCurve.transform(aTrsf), "Standard_NotImplemented");

    // Copy and original remain identical.
    let aU: f64 = PI / 4.0;
    let aOrigPt: Point2 = aCurve.eval_d0(aU).unwrap();
    let aCopyPt: Point2 = aCopyCurve.eval_d0(aU).unwrap();
    EXPECT_NEAR(aOrigPt.distance(aCopyPt), 0.0, precision::confusion());
}

// Test DumpJson does not crash
#[test]
fn Geom2dEval_TBezierCurveTest__DumpJson_NoCrash() {
    let aCurve = createSimpleCurve();
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}

// Test DumpJson does not crash for rational curve
#[test]
fn Geom2dEval_TBezierCurveTest__DumpJson_Rational_NoCrash() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    let aWeights = weights(&[1.0, 2.0, 1.0]);
    let aCurve = Geom2dEvalTBezierCurve::with_weights(&aPoles, &aWeights, 1.0).unwrap();
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}

// Test periodicity and continuity
#[test]
fn Geom2dEval_TBezierCurveTest__PeriodicityAndContinuity() {
    let aCurve = createSimpleCurve();
    assert!(!aCurve.is_periodic());
    assert_eq!(aCurve.continuity(), Shape::CN);
    assert!(aCurve.is_cn(0).unwrap());
    assert!(aCurve.is_cn(10).unwrap());
}

#[test]
fn Geom2dEval_TBezierCurveTest__Reverse_NotImplemented() {
    let aCurve = createSimpleCurve();
    EXPECT_THROW(aCurve.reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.reversed_parameter(0.5), "Standard_NotImplemented");
}

// Test EvalD1 derivative consistency for rational curve
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD1_Rational_ConsistentWithD0() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    let aWeights = weights(&[1.0, 3.0, 1.0]);
    let aCurve = Geom2dEvalTBezierCurve::with_weights(&aPoles, &aWeights, 1.0).unwrap();

    let aU: f64 = PI / 3.0;

    let aD1: Geom2dCurveD1 = aCurve.eval_d1(aU).unwrap();
    let aP1: Point2 = aCurve.eval_d0(aU + precision::confusion()).unwrap();
    let aP2: Point2 = aCurve.eval_d0(aU - precision::confusion()).unwrap();

    let aFD = Vector2::from((aP1.xy() - aP2.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD1.d1.x(), aFD.x(), THE_FD_TOL_D1);
    EXPECT_NEAR(aD1.d1.y(), aFD.y(), THE_FD_TOL_D1);
}

// Test EvalD2 consistency: compare D2 vs finite difference of D1
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD2_ConsistentWithD1() {
    let aCurve = createSimpleCurve();
    let aU: f64 = PI / 4.0;

    let aD2: Geom2dCurveD2 = aCurve.eval_d2(aU).unwrap();
    let aD1Plus: Geom2dCurveD1 = aCurve.eval_d1(aU + precision::confusion()).unwrap();
    let aD1Minus: Geom2dCurveD1 = aCurve.eval_d1(aU - precision::confusion()).unwrap();

    let aFD = Vector2::from((aD1Plus.d1.xy() - aD1Minus.d1.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD2.d2.x(), aFD.x(), THE_FD_TOL_D2);
    EXPECT_NEAR(aD2.d2.y(), aFD.y(), THE_FD_TOL_D2);
}

// Test EvalD3 consistency: compare D3 vs finite difference of D2
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD3_ConsistentWithD2() {
    let aCurve = createSimpleCurve();
    let aU: f64 = PI / 3.0;

    let aD3: Geom2dCurveD3 = aCurve.eval_d3(aU).unwrap();
    let aD2Plus: Geom2dCurveD2 = aCurve.eval_d2(aU + precision::confusion()).unwrap();
    let aD2Minus: Geom2dCurveD2 = aCurve.eval_d2(aU - precision::confusion()).unwrap();

    let aFD = Vector2::from((aD2Plus.d2.xy() - aD2Minus.d2.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD3.d3.x(), aFD.x(), THE_FD_TOL_D3);
    EXPECT_NEAR(aD3.d3.y(), aFD.y(), THE_FD_TOL_D3);
}

// Test EvalDN(u,1) matches D1 from EvalD1
#[test]
fn Geom2dEval_TBezierCurveTest__EvalDN_ConsistentWithD1() {
    let aCurve = createSimpleCurve();
    let aU: f64 = PI / 6.0;

    let aD1: Geom2dCurveD1 = aCurve.eval_d1(aU).unwrap();
    let aDN: Vector2 = aCurve.eval_dn(aU, 1).unwrap();

    EXPECT_NEAR(aD1.d1.x(), aDN.x(), precision::confusion());
    EXPECT_NEAR(aD1.d1.y(), aDN.y(), precision::confusion());
}

// Test EvalD0 at multiple evenly-spaced points produces finite results
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD0_MultiplePoints() {
    let aCurve = createSimpleCurve();
    let aFirst: f64 = aCurve.first_parameter();
    let aLast: f64 = aCurve.last_parameter();
    let aNbPoints: i32 = 6;

    for i in 0..aNbPoints {
        let aU: f64 = aFirst + (aLast - aFirst) * f64::from(i) / f64::from(aNbPoints - 1);
        let aPt: Point2 = aCurve.eval_d0(aU).unwrap();
        assert!(aPt.x().is_finite(), "Non-finite X at i={i}");
        assert!(aPt.y().is_finite(), "Non-finite Y at i={i}");
    }
}

// Test 2D T-Bezier semicircle matches Geom2d_Circle D0/D1/D2/D3
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD3_MatchesCircle() {
    let aTBez = createSemicircle2d();
    let aCircle = Handle::new(Geom2dCircle::new(Axis22d::default(), 1.0).unwrap());

    let aParams: [f64; 4] = [PI / 6.0, PI / 3.0, PI / 2.0, 2.0 * PI / 3.0];
    for aU in aParams {
        let aDT: Geom2dCurveD3 = aTBez.eval_d3(aU).unwrap();
        let aDC: Geom2dCurveD3 = aCircle.eval_d3(aU).unwrap();

        EXPECT_NEAR(aDT.point.distance(aDC.point), 0.0, precision::angular());
        EXPECT_NEAR(aDT.d1.x(), aDC.d1.x(), precision::angular());
        EXPECT_NEAR(aDT.d1.y(), aDC.d1.y(), precision::angular());
        EXPECT_NEAR(aDT.d2.x(), aDC.d2.x(), precision::angular());
        EXPECT_NEAR(aDT.d2.y(), aDC.d2.y(), precision::angular());
        EXPECT_NEAR(aDT.d3.x(), aDC.d3.x(), precision::angular());
        EXPECT_NEAR(aDT.d3.y(), aDC.d3.y(), precision::angular());
    }
}

// Test 2D T-Bezier semi-ellipse matches Geom2d_Ellipse D0/D1/D2
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD2_MatchesEllipse() {
    let aTBez = createSemiEllipse2d();
    let anEllipse = Handle::new(Geom2dEllipse::new(Axis22d::default(), 3.0, 2.0).unwrap());

    let aParams: [f64; 4] = [PI / 6.0, PI / 4.0, PI / 2.0, 3.0 * PI / 4.0];
    for aU in aParams {
        let aDT: Geom2dCurveD2 = aTBez.eval_d2(aU).unwrap();
        let aDC: Geom2dCurveD2 = anEllipse.eval_d2(aU).unwrap();

        EXPECT_NEAR(aDT.point.distance(aDC.point), 0.0, precision::angular());
        EXPECT_NEAR(aDT.d1.x(), aDC.d1.x(), precision::angular());
        EXPECT_NEAR(aDT.d1.y(), aDC.d1.y(), precision::angular());
        EXPECT_NEAR(aDT.d2.x(), aDC.d2.x(), precision::angular());
        EXPECT_NEAR(aDT.d2.y(), aDC.d2.y(), precision::angular());
    }
}
