// Test Copy produces independent identical object
#[test]
fn Geom2dEval_AHTBezierCurveTest__Copy_Independent() {
    let aCurve = createFullBasisCurve();
    let aCopy = aCurve.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyCurve: Handle<Geom2dEvalAHTBezierCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());
    assert_eq!(aCopyCurve.nb_poles(), 6);
    assert_eq!(aCopyCurve.alg_degree(), 1);

    // Verify the copy evaluates identically at several parameters
    let mut aU: f64 = 0.0;
    while aU <= 1.0 {
        let aOrigPnt: Point2 = aCurve.eval_d0(aU).unwrap();
        let aCopyPnt: Point2 = aCopyCurve.eval_d0(aU).unwrap();
        EXPECT_NEAR(aOrigPnt.distance(aCopyPnt), 0.0, precision::confusion());
        aU += 0.25;
    }

    // Verify behavior: transformation is not supported on the copy either.
    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(10.0, 10.0));
    let aCopyHandle: Handle<Geom2dEvalAHTBezierCurve> = aCopy.cast();
    EXPECT_THROW(aCopyHandle.transform(aTrsf), "Standard_NotImplemented");

    let aOrigMid: Point2 = aCurve.eval_d0(0.5).unwrap();
    let aCopyMid: Point2 = aCopyHandle.eval_d0(0.5).unwrap();
    EXPECT_NEAR(aOrigMid.distance(aCopyMid), 0.0, precision::confusion());
}

// Test Copy of rational curve preserves rationality
#[test]
fn Geom2dEval_AHTBezierCurveTest__Copy_Rational() {
    let aCurve = createRationalCurve();
    let aCopy = aCurve.copy().unwrap();
    let aCopyCurve: Handle<Geom2dEvalAHTBezierCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());
    assert!(aCopyCurve.is_rational());
}

// Test DumpJson does not crash
#[test]
fn Geom2dEval_AHTBezierCurveTest__DumpJson_NoCrash() {
    let aCurve = createFullBasisCurve();
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}

// Test DumpJson on rational curve does not crash
#[test]
fn Geom2dEval_AHTBezierCurveTest__DumpJson_Rational_NoCrash() {
    let aCurve = createRationalCurve();
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}

// Test invalid construction: pole count mismatch throws
#[test]
fn Geom2dEval_AHTBezierCurveTest__Construction_PoleCountMismatch_Throws() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
    // algDegree=1, alpha=1.0, beta=1.0 => basisDim = 2+2+2 = 6, but only 4 poles
    EXPECT_THROW(
        Geom2dEvalAHTBezierCurve::new(&aPoles, 1, 1.0, 1.0),
        "Standard_ConstructionError",
    );
}

// Test invalid construction: negative alpha throws
#[test]
fn Geom2dEval_AHTBezierCurveTest__Construction_NegativeAlpha_Throws() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)]);
    EXPECT_THROW(
        Geom2dEvalAHTBezierCurve::new(&aPoles, 2, -1.0, 0.0),
        "Standard_ConstructionError",
    );
}

// Test invalid construction: negative beta throws
#[test]
fn Geom2dEval_AHTBezierCurveTest__Construction_NegativeBeta_Throws() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)]);
    EXPECT_THROW(
        Geom2dEvalAHTBezierCurve::new(&aPoles, 2, 0.0, -1.0),
        "Standard_ConstructionError",
    );
}

// Test invalid construction: weight count mismatch throws
#[test]
fn Geom2dEval_AHTBezierCurveTest__Construction_WeightCountMismatch_Throws() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    let aWeights = weights(&[1.0, 1.0]);
    EXPECT_THROW(
        Geom2dEvalAHTBezierCurve::with_weights(&aPoles, &aWeights, 2, 0.0, 0.0),
        "Standard_ConstructionError",
    );
}

// Test invalid construction: non-positive weight throws
#[test]
fn Geom2dEval_AHTBezierCurveTest__Construction_NonPositiveWeight_Throws() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    let aWeights = weights(&[1.0, -1.0, 1.0]);
    EXPECT_THROW(
        Geom2dEvalAHTBezierCurve::with_weights(&aPoles, &aWeights, 2, 0.0, 0.0),
        "Standard_ConstructionError",
    );
}

// Reverse semantics are intentionally unsupported for this evaluator.
#[test]
fn Geom2dEval_AHTBezierCurveTest__Reverse_NotImplemented() {
    let aCurve = createFullBasisCurve();
    EXPECT_THROW(aCurve.reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.reversed_parameter(0.5), "Standard_NotImplemented");
}

// Test EvalD2 consistency: compare D2 vs finite difference of D1
#[test]
fn Geom2dEval_AHTBezierCurveTest__EvalD2_ConsistentWithD1() {
    let aCurve = createFullBasisCurve();
    let aU: f64 = 0.3;

    let aD2: Geom2dCurveD2 = aCurve.eval_d2(aU).unwrap();
    let aD1Plus: Geom2dCurveD1 = aCurve.eval_d1(aU + precision::confusion()).unwrap();
    let aD1Minus: Geom2dCurveD1 = aCurve.eval_d1(aU - precision::confusion()).unwrap();

    let aFD = Vector2::from((aD1Plus.d1.xy() - aD1Minus.d1.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD2.d2.x(), aFD.x(), THE_FD_TOL_D2);
    EXPECT_NEAR(aD2.d2.y(), aFD.y(), THE_FD_TOL_D2);
}

// Test EvalD3 consistency: compare D3 vs finite difference of D2
#[test]
fn Geom2dEval_AHTBezierCurveTest__EvalD3_ConsistentWithD2() {
    let aCurve = createFullBasisCurve();
    let aU: f64 = 0.4;

    let aD3: Geom2dCurveD3 = aCurve.eval_d3(aU).unwrap();
    let aD2Plus: Geom2dCurveD2 = aCurve.eval_d2(aU + precision::confusion()).unwrap();
    let aD2Minus: Geom2dCurveD2 = aCurve.eval_d2(aU - precision::confusion()).unwrap();

    let aFD = Vector2::from((aD2Plus.d2.xy() - aD2Minus.d2.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD3.d3.x(), aFD.x(), THE_FD_TOL_D3);
    EXPECT_NEAR(aD3.d3.y(), aFD.y(), THE_FD_TOL_D3);
}

// Test EvalDN(u,1) matches D1 from EvalD1
#[test]
fn Geom2dEval_AHTBezierCurveTest__EvalDN_ConsistentWithD1() {
    let aCurve = createFullBasisCurve();
    let aU: f64 = 0.5;

    let aD1: Geom2dCurveD1 = aCurve.eval_d1(aU).unwrap();
    let aDN: Vector2 = aCurve.eval_dn(aU, 1).unwrap();

    EXPECT_NEAR(aD1.d1.x(), aDN.x(), precision::confusion());
    EXPECT_NEAR(aD1.d1.y(), aDN.y(), precision::confusion());
}

// Test EvalD0 for pure polynomial line: alpha=0, beta=0, algDegree=1, basis {1, t}
// P(t) = (0,0)*1 + (1,1)*t => P(0.5) = (0.5, 0.5), P(1) = (1, 1)
#[test]
fn Geom2dEval_AHTBezierCurveTest__EvalD0_PurePolynomial_Line() {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0)]);
    let aCurve = Geom2dEvalAHTBezierCurve::new(&aPoles, 1, 0.0, 0.0).unwrap();

    let aPtHalf: Point2 = aCurve.eval_d0(0.5).unwrap();
    EXPECT_NEAR(aPtHalf.x(), 0.5, precision::confusion());
    EXPECT_NEAR(aPtHalf.y(), 0.5, precision::confusion());

    let aPtEnd: Point2 = aCurve.eval_d0(1.0).unwrap();
    EXPECT_NEAR(aPtEnd.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aPtEnd.y(), 1.0, precision::confusion());
}
