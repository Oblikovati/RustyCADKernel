// ============================================================================
// Axis Orientation Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Circle_DifferentAxisOrientation_DifferentHash() {
    let myHasher = SetUp();
    let anAxis1 = ax2(pnt(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let anAxis2 = ax2(pnt(0.0, 0.0, 0.0), dir(0.0, 1.0, 0.0));
    let aCircle1 = circle(anAxis1, 5.0);
    let aCircle2 = circle(anAxis2, 5.0);

    assert_ne!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(!myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

#[test]
fn GeomHash_CurveHasherTest__Line_DifferentDirection_DifferentHash() {
    let myHasher = SetUp();
    let aLine1 = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aLine2 = line(pnt(0.0, 0.0, 0.0), dir(0.0, 1.0, 0.0));

    assert_ne!(myHasher.hash(&curve(&aLine1)), myHasher.hash(&curve(&aLine2)));
    assert!(!myHasher.equal(&curve(&aLine1), &curve(&aLine2)));
}

// ============================================================================
// BSpline with Different Knots Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__BSplineCurve_DifferentKnots_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles = poles(&[
        (0.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (2.0, 1.0, 0.0),
        (3.0, 0.0, 0.0),
    ]);
    let aKnots1 = reals(&[0.0, 1.0]);
    // The last knot differs.
    let aKnots2 = reals(&[0.0, 2.0]);
    let aMults = integers(&[4, 4]);

    let aCurve1 = bspline(&aPoles, &aKnots1, &aMults, 3);
    let aCurve2 = bspline(&aPoles, &aKnots2, &aMults, 3);

    assert!(!myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Periodic BSpline Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__BSplineCurve_QuadraticBezierEquivalent_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    // Simple quadratic B-spline (Bezier-like, single span)
    let aPoles = poles(&[(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, 0.0, 0.0)]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = integers(&[3, 3]);

    let aCurve1 = bspline(&aPoles, &aKnots, &aMults, 2);
    let aCurve2: Handle<GeomBSplineCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Reversed Curve Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Line_Reversed_DifferentComparison() {
    let myHasher = SetUp();
    let aLine1 = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aLine2: Handle<GeomLine> = aLine1.reversed().unwrap().cast();

    // Reversed line has opposite direction
    assert!(!myHasher.equal(&curve(&aLine1), &curve(&aLine2)));
}

#[test]
fn GeomHash_CurveHasherTest__BezierCurve_Reversed_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles = poles(&[(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, 0.0, 0.0)]);

    let aCurve1 = Handle::new(GeomBezierCurve::new(&aPoles).unwrap());
    let aCurve2: Handle<GeomBezierCurve> = aCurve1.reversed().unwrap().cast();

    // Reversed curve has poles in different order
    assert!(!myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Transformed Curve Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Circle_Translated_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCircle1 = circle(anAxis, 5.0);
    let aCircle2: Handle<GeomCircle> = copied(&aCircle1);
    aCircle2.translate(Vector3::new(1.0, 0.0, 0.0)).unwrap();

    assert_ne!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(!myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

#[test]
fn GeomHash_CurveHasherTest__Circle_Scaled_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCircle1 = circle(anAxis, 5.0);
    let aCircle2: Handle<GeomCircle> = copied(&aCircle1);
    aCircle2.scale(pnt(0.0, 0.0, 0.0), 2.0).unwrap();

    assert_ne!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(!myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

// ============================================================================
// Higher Degree BSpline Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__BSplineCurve_HigherDegree_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    let aPoles = poles(&[
        (0.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (2.0, 0.5, 0.0),
        (3.0, 1.5, 0.0),
        (4.0, 0.0, 0.0),
        (5.0, 1.0, 0.0),
    ]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = integers(&[6, 6]);

    // Degree 5 curve
    let aCurve1 = bspline(&aPoles, &aKnots, &aMults, 5);
    let aCurve2: Handle<GeomBSplineCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

#[test]
fn GeomHash_CurveHasherTest__BSplineCurve_DifferentDegree_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles = poles(&[
        (0.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (2.0, 1.0, 0.0),
        (3.0, 0.0, 0.0),
    ]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults1 = integers(&[4, 4]);
    let aMults2 = integers(&[3, 3]);

    // Different degrees: 3 vs 2
    let aCurve1 = bspline(&aPoles, &aKnots, &aMults1, 3);

    let aPoles2 = poles(&[(0.0, 0.0, 0.0), (1.5, 1.0, 0.0), (3.0, 0.0, 0.0)]);

    let aCurve2 = bspline(&aPoles2, &aKnots, &aMults2, 2);

    assert!(!myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Multiple Knot Spans Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__BSplineCurve_LinearMultipleSpans_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    // Linear B-spline with multiple knots (piecewise linear)
    let aPoles = poles(&[
        (0.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (2.0, 0.0, 0.0),
        (3.0, 1.0, 0.0),
    ]);
    let aKnots = reals(&[0.0, 1.0, 2.0, 3.0]);
    let aMults = integers(&[2, 1, 1, 2]);

    let aCurve1 = bspline(&aPoles, &aKnots, &aMults, 1);
    let aCurve2: Handle<GeomBSplineCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Cross-type Conic Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Ellipse_vs_Hyperbola_DifferentComparison() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let anEllipse = Handle::new(GeomEllipse::new(anAxis, 5.0, 3.0).unwrap());
    let aHyperbola = Handle::new(GeomHyperbola::new(anAxis, 5.0, 3.0).unwrap());

    assert!(!myHasher.equal(&curve(&anEllipse), &curve(&aHyperbola)));
}

#[test]
fn GeomHash_CurveHasherTest__Circle_vs_Ellipse_DifferentComparison() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCircle = circle(anAxis, 5.0);
    let anEllipse = Handle::new(GeomEllipse::new(anAxis, 5.0, 5.0).unwrap());

    assert!(!myHasher.equal(&curve(&aCircle), &curve(&anEllipse)));
}

// ============================================================================
// Trimmed vs Base Curve Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__TrimmedCurve_vs_BaseCurve_DifferentComparison() {
    let myHasher = SetUp();
    let aLine = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aTrimmed =
        Handle::new(GeomTrimmedCurve::new(&curve(&aLine), 0.0, 10.0, true, true).unwrap());

    // Trimmed curve is a different type from base curve
    assert!(!myHasher.equal(&curve(&aLine), &curve(&aTrimmed)));
}

// ============================================================================
// Edge Cases - Degenerate/Special Values
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Circle_VerySmallRadius_CopiedCircles_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCircle1 = circle(anAxis, 1e-10);
    let aCircle2: Handle<GeomCircle> = copied(&aCircle1);

    assert_eq!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

#[test]
fn GeomHash_CurveHasherTest__Circle_VeryLargeRadius_CopiedCircles_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCircle1 = circle(anAxis, 1e10);
    let aCircle2: Handle<GeomCircle> = copied(&aCircle1);

    assert_eq!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

#[test]
fn GeomHash_CurveHasherTest__Line_AtOrigin_vs_FarFromOrigin_DifferentHash() {
    let myHasher = SetUp();
    let aLine1 = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aLine2 = line(pnt(1e10, 1e10, 1e10), dir(1.0, 0.0, 0.0));

    assert_ne!(myHasher.hash(&curve(&aLine1)), myHasher.hash(&curve(&aLine2)));
    assert!(!myHasher.equal(&curve(&aLine1), &curve(&aLine2)));
}
