#[test]
fn Geom2dHash_CurveHasherTest__BSplineCurve_DifferentWeights_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 1.0), (3.0, 0.0)]);
    let aWeights1 = reals(&[1.0, 2.0, 2.0, 1.0]);
    // Different second weight
    let aWeights2 = reals(&[1.0, 3.0, 2.0, 1.0]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = integers(&[4, 4]);

    let aCurve1 = Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeights1, &aKnots, &aMults, 3, false).unwrap(),
    );
    let aCurve2 = Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeights2, &aKnots, &aMults, 3, false).unwrap(),
    );

    assert!(!myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Weighted Bezier Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__BezierCurve_Weighted_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    let aPoles = poles(&[(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)]);
    let aWeights = reals(&[1.0, 2.0, 1.0]);

    let aCurve1 = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aCurve2: Handle<Geom2dBezierCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Axis Orientation Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__Circle_DifferentAxisOrientation_DifferentHash() {
    let myHasher = SetUp();
    let anAxis1 = axes(dir(1.0, 0.0), dir(0.0, 1.0));
    let anAxis2 = axes(dir(0.0, 1.0), dir(-1.0, 0.0));
    let aCircle1 = circle(anAxis1, 5.0);
    let aCircle2 = circle(anAxis2, 5.0);

    assert_ne!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(!myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

#[test]
fn Geom2dHash_CurveHasherTest__Line_DifferentDirection_DifferentHash() {
    let myHasher = SetUp();
    let aLine1 = line(pnt(0.0, 0.0), dir(1.0, 0.0));
    let aLine2 = line(pnt(0.0, 0.0), dir(0.0, 1.0));

    assert_ne!(myHasher.hash(&curve(&aLine1)), myHasher.hash(&curve(&aLine2)));
    assert!(!myHasher.equal(&curve(&aLine1), &curve(&aLine2)));
}

// ============================================================================
// BSpline with Different Knots Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__BSplineCurve_DifferentKnots_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 1.0), (3.0, 0.0)]);
    let aKnots1 = reals(&[0.0, 1.0]);
    // Different last knot
    let aKnots2 = reals(&[0.0, 2.0]);
    let aMults = integers(&[4, 4]);

    let aCurve1 =
        Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots1, &aMults, 3, false).unwrap());
    let aCurve2 =
        Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots2, &aMults, 3, false).unwrap());

    assert!(!myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Periodic BSpline Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__BSplineCurve_QuadraticBezierEquivalent_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    // Simple quadratic B-spline (Bezier-like, single span)
    let aPoles = poles(&[(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = integers(&[3, 3]);

    let aCurve1 =
        Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap());
    let aCurve2: Handle<Geom2dBSplineCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Reversed Curve Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__Line_Reversed_DifferentComparison() {
    let myHasher = SetUp();
    let aLine1 = line(pnt(0.0, 0.0), dir(1.0, 0.0));
    let aLine2: Handle<Geom2dLine> = aLine1.reversed().unwrap().cast();

    assert!(!myHasher.equal(&curve(&aLine1), &curve(&aLine2)));
}

#[test]
fn Geom2dHash_CurveHasherTest__BezierCurve_Reversed_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles = poles(&[(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)]);

    let aCurve1 = Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap());
    let aCurve2: Handle<Geom2dBezierCurve> = aCurve1.reversed().unwrap().cast();

    assert!(!myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Transformed Curve Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__Circle_Translated_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axes();
    let aCircle1 = circle(anAxis, 5.0);
    let aCircle2: Handle<Geom2dCircle> = copied(&aCircle1);
    aCircle2.translate(Vector2::new(1.0, 0.0)).unwrap();

    assert_ne!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(!myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

#[test]
fn Geom2dHash_CurveHasherTest__Circle_Scaled_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axes();
    let aCircle1 = circle(anAxis, 5.0);
    let aCircle2: Handle<Geom2dCircle> = copied(&aCircle1);
    aCircle2.scale(pnt(0.0, 0.0), 2.0).unwrap();

    assert_ne!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(!myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

// ============================================================================
// Higher Degree BSpline Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__BSplineCurve_HigherDegree_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    let aPoles = poles(&[
        (0.0, 0.0),
        (1.0, 1.0),
        (2.0, 0.5),
        (3.0, 1.5),
        (4.0, 0.0),
        (5.0, 1.0),
    ]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = integers(&[6, 6]);

    let aCurve1 =
        Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 5, false).unwrap());
    let aCurve2: Handle<Geom2dBSplineCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Multiple Knot Spans Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__BSplineCurve_LinearMultipleSpans_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    // Linear B-spline with multiple knots (piecewise linear)
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0)]);
    let aKnots = reals(&[0.0, 1.0, 2.0, 3.0]);
    let aMults = integers(&[2, 1, 1, 2]);

    let aCurve1 =
        Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 1, false).unwrap());
    let aCurve2: Handle<Geom2dBSplineCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Cross-type Conic Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__Ellipse_vs_Hyperbola_DifferentComparison() {
    let myHasher = SetUp();
    let anAxis = standard_axes();
    let anEllipse = Handle::new(Geom2dEllipse::new(anAxis, 5.0, 3.0).unwrap());
    let aHyperbola = Handle::new(Geom2dHyperbola::new(anAxis, 5.0, 3.0).unwrap());

    assert!(!myHasher.equal(&curve(&anEllipse), &curve(&aHyperbola)));
}

#[test]
fn Geom2dHash_CurveHasherTest__Circle_vs_Ellipse_DifferentComparison() {
    let myHasher = SetUp();
    let anAxis = standard_axes();
    let aCircle = circle(anAxis, 5.0);
    let anEllipse = Handle::new(Geom2dEllipse::new(anAxis, 5.0, 5.0).unwrap());

    assert!(!myHasher.equal(&curve(&aCircle), &curve(&anEllipse)));
}

// ============================================================================
// Trimmed vs Base Curve Tests
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__TrimmedCurve_vs_BaseCurve_DifferentComparison() {
    let myHasher = SetUp();
    let aLine = line(pnt(0.0, 0.0), dir(1.0, 0.0));
    let aTrimmed =
        Handle::new(Geom2dTrimmedCurve::new(&curve(&aLine), 0.0, 10.0, true, true).unwrap());

    assert!(!myHasher.equal(&curve(&aLine), &curve(&aTrimmed)));
}

// ============================================================================
// Edge Cases - Degenerate/Special Values
// ============================================================================

#[test]
fn Geom2dHash_CurveHasherTest__Circle_VerySmallRadius_CopiedCircles_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axes();
    let aCircle1 = circle(anAxis, 1e-10);
    let aCircle2: Handle<Geom2dCircle> = copied(&aCircle1);

    assert_eq!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

#[test]
fn Geom2dHash_CurveHasherTest__Circle_VeryLargeRadius_CopiedCircles_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axes();
    let aCircle1 = circle(anAxis, 1e10);
    let aCircle2: Handle<Geom2dCircle> = copied(&aCircle1);

    assert_eq!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

#[test]
fn Geom2dHash_CurveHasherTest__Line_AtOrigin_vs_FarFromOrigin_DifferentHash() {
    let myHasher = SetUp();
    let aLine1 = line(pnt(0.0, 0.0), dir(1.0, 0.0));
    let aLine2 = line(pnt(1e10, 1e10), dir(1.0, 0.0));

    assert_ne!(myHasher.hash(&curve(&aLine1)), myHasher.hash(&curve(&aLine2)));
    assert!(!myHasher.equal(&curve(&aLine1), &curve(&aLine2)));
}
