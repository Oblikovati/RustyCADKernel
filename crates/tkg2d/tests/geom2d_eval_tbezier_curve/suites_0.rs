// Test construction with valid parameters (linear, order 1)
#[test]
fn Geom2dEval_TBezierCurveTest__Construction_Linear() {
    let aCurve = createSimpleCurve();
    assert_eq!(aCurve.nb_poles(), 3);
    assert_eq!(aCurve.order(), 1);
    EXPECT_NEAR(aCurve.alpha(), 1.0, precision::confusion());
    assert!(!aCurve.is_rational());
}

// Test construction with valid parameters (quadratic, order 2)
#[test]
fn Geom2dEval_TBezierCurveTest__Construction_Quadratic() {
    let aCurve = createQuadraticCurve();
    assert_eq!(aCurve.nb_poles(), 5);
    assert_eq!(aCurve.order(), 2);
    EXPECT_NEAR(aCurve.alpha(), 0.5, precision::confusion());
    assert!(!aCurve.is_rational());
}

// Test construction throws for invalid parameters
#[test]
fn Geom2dEval_TBezierCurveTest__Construction_InvalidParams_Throws() {
    // Even number of poles (not odd)
    let aPoles2 = poles(&[(0.0, 0.0), (1.0, 0.0)]);
    EXPECT_THROW(
        Geom2dEvalTBezierCurve::new(&aPoles2, 1.0),
        "Standard_ConstructionError",
    );

    // Too few poles
    let aPoles1 = poles(&[(0.0, 0.0)]);
    EXPECT_THROW(
        Geom2dEvalTBezierCurve::new(&aPoles1, 1.0),
        "Standard_ConstructionError",
    );

    // Invalid alpha
    let aPoles3 = poles(&[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)]);
    EXPECT_THROW(
        Geom2dEvalTBezierCurve::new(&aPoles3, 0.0),
        "Standard_ConstructionError",
    );
    EXPECT_THROW(
        Geom2dEvalTBezierCurve::new(&aPoles3, -1.0),
        "Standard_ConstructionError",
    );
}

// Test EvalD0 at endpoints matches StartPoint/EndPoint
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD0_Endpoints() {
    let aCurve = createSimpleCurve();

    let aStart: Point2 = aCurve.eval_d0(aCurve.first_parameter()).unwrap();
    let aEnd: Point2 = aCurve.eval_d0(aCurve.last_parameter()).unwrap();

    EXPECT_NEAR(
        aStart.distance(aCurve.start_point().unwrap()),
        0.0,
        precision::confusion(),
    );
    EXPECT_NEAR(
        aEnd.distance(aCurve.end_point().unwrap()),
        0.0,
        precision::confusion(),
    );
}

// Test EvalD0 at a known midpoint for the quadratic curve
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD0_Midpoint() {
    let aCurve = createSimpleCurve();
    let aMid: f64 = (aCurve.first_parameter() + aCurve.last_parameter()) / 2.0;
    let aPt: Point2 = aCurve.eval_d0(aMid).unwrap();

    // Verify the point is finite and within a reasonable range
    assert!(aPt.x().is_finite());
    assert!(aPt.y().is_finite());
}

// Test derivative consistency: compare EvalD1 vs finite difference of EvalD0
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD1_ConsistentWithD0() {
    let aCurve = createSimpleCurve();
    let aU: f64 = PI / 3.0;

    let aD1: Geom2dCurveD1 = aCurve.eval_d1(aU).unwrap();
    let aP1: Point2 = aCurve.eval_d0(aU + precision::confusion()).unwrap();
    let aP2: Point2 = aCurve.eval_d0(aU - precision::confusion()).unwrap();

    let aFD = Vector2::from((aP1.xy() - aP2.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD1.d1.x(), aFD.x(), THE_FD_TOL_D1);
    EXPECT_NEAR(aD1.d1.y(), aFD.y(), THE_FD_TOL_D1);
}

// Test derivative consistency for the quadratic curve
#[test]
fn Geom2dEval_TBezierCurveTest__EvalD1_QuadraticConsistentWithD0() {
    let aCurve = createQuadraticCurve();
    let aU: f64 = PI / 4.0;

    let aD1: Geom2dCurveD1 = aCurve.eval_d1(aU).unwrap();
    let aP1: Point2 = aCurve.eval_d0(aU + precision::confusion()).unwrap();
    let aP2: Point2 = aCurve.eval_d0(aU - precision::confusion()).unwrap();

    let aFD = Vector2::from((aP1.xy() - aP2.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD1.d1.x(), aFD.x(), THE_FD_TOL_D1);
    EXPECT_NEAR(aD1.d1.y(), aFD.y(), THE_FD_TOL_D1);
}

// Test Bounds returns correct domain [0, Pi/alpha]
#[test]
fn Geom2dEval_TBezierCurveTest__Bounds() {
    let aCurve = createSimpleCurve();
    EXPECT_NEAR(aCurve.first_parameter(), 0.0, precision::confusion());
    EXPECT_NEAR(aCurve.last_parameter(), PI / 1.0, precision::confusion());

    let aCurve2 = createQuadraticCurve();
    EXPECT_NEAR(aCurve2.first_parameter(), 0.0, precision::confusion());
    EXPECT_NEAR(aCurve2.last_parameter(), PI / 0.5, precision::confusion());
}

// Test IsRational returns false for non-rational curve
#[test]
fn Geom2dEval_TBezierCurveTest__IsRational_NonRational() {
    let aCurve = createSimpleCurve();
    assert!(!aCurve.is_rational());
}

// Test IsRational returns true for rational curve with weights
#[test]
fn Geom2dEval_TBezierCurveTest__IsRational_WithWeights() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    let aWeights = weights(&[1.0, 2.0, 1.0]);
    let aCurve = Geom2dEvalTBezierCurve::with_weights(&aPoles, &aWeights, 1.0).unwrap();
    assert!(aCurve.is_rational());
    assert_eq!(aCurve.weights().len(), 3);
}

// Test rational construction throws for mismatched weights size
#[test]
fn Geom2dEval_TBezierCurveTest__Construction_Rational_MismatchedWeights_Throws() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 5).unwrap();
    for i in 1..=5 {
        aWeights.set_value(i, 1.0).unwrap();
    }
    EXPECT_THROW(
        Geom2dEvalTBezierCurve::with_weights(&aPoles, &aWeights, 1.0),
        "Standard_ConstructionError",
    );
}

// Test rational construction throws for non-positive weight
#[test]
fn Geom2dEval_TBezierCurveTest__Construction_Rational_NegativeWeight_Throws() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    let aWeights = weights(&[1.0, -1.0, 1.0]);
    EXPECT_THROW(
        Geom2dEvalTBezierCurve::with_weights(&aPoles, &aWeights, 1.0),
        "Standard_ConstructionError",
    );
}

// Test NbPoles and Alpha accessors
#[test]
fn Geom2dEval_TBezierCurveTest__NbPoles_Alpha() {
    let aCurve = createSimpleCurve();
    assert_eq!(aCurve.nb_poles(), 3);
    EXPECT_NEAR(aCurve.alpha(), 1.0, precision::confusion());

    let aCurve2 = createQuadraticCurve();
    assert_eq!(aCurve2.nb_poles(), 5);
    EXPECT_NEAR(aCurve2.alpha(), 0.5, precision::confusion());
}

// Test Transform/Transformed are not implemented
#[test]
fn Geom2dEval_TBezierCurveTest__Transform_NotImplemented() {
    let aCurve = createSimpleCurve();
    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(1.0, 2.0));

    EXPECT_THROW(aCurve.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.transformed(aTrsf), "Standard_NotImplemented");
}
