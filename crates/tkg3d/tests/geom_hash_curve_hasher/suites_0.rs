// ============================================================================
// Line Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Line_CopiedLines_SameHash() {
    let myHasher = SetUp();
    let aLoc = pnt(1.0, 2.0, 3.0);
    let aDir = dir(1.0, 0.0, 0.0);
    let aLine1 = line(aLoc, aDir);
    let aLine2: Handle<GeomLine> = copied(&aLine1);

    assert_eq!(myHasher.hash(&curve(&aLine1)), myHasher.hash(&curve(&aLine2)));
    assert!(myHasher.equal(&curve(&aLine1), &curve(&aLine2)));
}

#[test]
fn GeomHash_CurveHasherTest__Line_DifferentLines_DifferentHash() {
    let myHasher = SetUp();
    let aLine1 = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aLine2 = line(pnt(1.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));

    assert_ne!(myHasher.hash(&curve(&aLine1)), myHasher.hash(&curve(&aLine2)));
    assert!(!myHasher.equal(&curve(&aLine1), &curve(&aLine2)));
}

// ============================================================================
// Circle Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Circle_CopiedCircles_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCircle1 = circle(anAxis, 5.0);
    let aCircle2: Handle<GeomCircle> = copied(&aCircle1);

    assert_eq!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

#[test]
fn GeomHash_CurveHasherTest__Circle_DifferentRadius_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCircle1 = circle(anAxis, 5.0);
    let aCircle2 = circle(anAxis, 10.0);

    assert_ne!(myHasher.hash(&curve(&aCircle1)), myHasher.hash(&curve(&aCircle2)));
    assert!(!myHasher.equal(&curve(&aCircle1), &curve(&aCircle2)));
}

// ============================================================================
// Ellipse Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Ellipse_CopiedEllipses_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let anEllipse1 = Handle::new(GeomEllipse::new(anAxis, 10.0, 5.0).unwrap());
    let anEllipse2: Handle<GeomEllipse> = copied(&anEllipse1);

    assert_eq!(myHasher.hash(&curve(&anEllipse1)), myHasher.hash(&curve(&anEllipse2)));
    assert!(myHasher.equal(&curve(&anEllipse1), &curve(&anEllipse2)));
}

#[test]
fn GeomHash_CurveHasherTest__Ellipse_DifferentRadii_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let anEllipse1 = Handle::new(GeomEllipse::new(anAxis, 10.0, 5.0).unwrap());
    let anEllipse2 = Handle::new(GeomEllipse::new(anAxis, 10.0, 7.0).unwrap());

    assert_ne!(myHasher.hash(&curve(&anEllipse1)), myHasher.hash(&curve(&anEllipse2)));
    assert!(!myHasher.equal(&curve(&anEllipse1), &curve(&anEllipse2)));
}

// ============================================================================
// Hyperbola Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Hyperbola_CopiedHyperbolas_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aHyp1 = Handle::new(GeomHyperbola::new(anAxis, 5.0, 3.0).unwrap());
    let aHyp2: Handle<GeomHyperbola> = copied(&aHyp1);

    assert_eq!(myHasher.hash(&curve(&aHyp1)), myHasher.hash(&curve(&aHyp2)));
    assert!(myHasher.equal(&curve(&aHyp1), &curve(&aHyp2)));
}

#[test]
fn GeomHash_CurveHasherTest__Hyperbola_DifferentRadii_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aHyp1 = Handle::new(GeomHyperbola::new(anAxis, 5.0, 3.0).unwrap());
    let aHyp2 = Handle::new(GeomHyperbola::new(anAxis, 5.0, 4.0).unwrap());

    assert_ne!(myHasher.hash(&curve(&aHyp1)), myHasher.hash(&curve(&aHyp2)));
    assert!(!myHasher.equal(&curve(&aHyp1), &curve(&aHyp2)));
}

// ============================================================================
// Parabola Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__Parabola_CopiedParabolas_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aPar1 = Handle::new(GeomParabola::new(anAxis, 2.0).unwrap());
    let aPar2: Handle<GeomParabola> = copied(&aPar1);

    assert_eq!(myHasher.hash(&curve(&aPar1)), myHasher.hash(&curve(&aPar2)));
    assert!(myHasher.equal(&curve(&aPar1), &curve(&aPar2)));
}

#[test]
fn GeomHash_CurveHasherTest__Parabola_DifferentFocal_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aPar1 = Handle::new(GeomParabola::new(anAxis, 2.0).unwrap());
    let aPar2 = Handle::new(GeomParabola::new(anAxis, 3.0).unwrap());

    assert_ne!(myHasher.hash(&curve(&aPar1)), myHasher.hash(&curve(&aPar2)));
    assert!(!myHasher.equal(&curve(&aPar1), &curve(&aPar2)));
}

// ============================================================================
// BezierCurve Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__BezierCurve_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    let aPoles = poles(&[
        (0.0, 0.0, 0.0),
        (1.0, 2.0, 0.0),
        (3.0, 2.0, 0.0),
        (4.0, 0.0, 0.0),
    ]);

    let aCurve1 = Handle::new(GeomBezierCurve::new(&aPoles).unwrap());
    let aCurve2: Handle<GeomBezierCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

#[test]
fn GeomHash_CurveHasherTest__BezierCurve_DifferentPoles_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles1 = poles(&[(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, 0.0, 0.0)]);
    // The middle pole differs.
    let aPoles2 = poles(&[(0.0, 0.0, 0.0), (1.0, 3.0, 0.0), (2.0, 0.0, 0.0)]);

    let aCurve1 = Handle::new(GeomBezierCurve::new(&aPoles1).unwrap());
    let aCurve2 = Handle::new(GeomBezierCurve::new(&aPoles2).unwrap());

    // Hash may be same (metadata only), but comparison should differ
    assert!(!myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// BSplineCurve Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__BSplineCurve_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    let aPoles = poles(&[
        (0.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (2.0, 1.0, 0.0),
        (3.0, 0.0, 0.0),
    ]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = integers(&[4, 4]);

    let aCurve1 = bspline(&aPoles, &aKnots, &aMults, 3);
    let aCurve2: Handle<GeomBSplineCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// TrimmedCurve Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__TrimmedCurve_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    let aLine = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aTrimmed1 =
        Handle::new(GeomTrimmedCurve::new(&curve(&aLine), 0.0, 10.0, true, true).unwrap());
    let aTrimmed2: Handle<GeomTrimmedCurve> = copied(&aTrimmed1);

    assert_eq!(myHasher.hash(&curve(&aTrimmed1)), myHasher.hash(&curve(&aTrimmed2)));
    assert!(myHasher.equal(&curve(&aTrimmed1), &curve(&aTrimmed2)));
}

#[test]
fn GeomHash_CurveHasherTest__TrimmedCurve_DifferentBounds_DifferentHash() {
    let myHasher = SetUp();
    let aLine = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aTrimmed1 =
        Handle::new(GeomTrimmedCurve::new(&curve(&aLine), 0.0, 10.0, true, true).unwrap());
    let aTrimmed2 =
        Handle::new(GeomTrimmedCurve::new(&curve(&aLine), 0.0, 20.0, true, true).unwrap());

    assert_ne!(myHasher.hash(&curve(&aTrimmed1)), myHasher.hash(&curve(&aTrimmed2)));
    assert!(!myHasher.equal(&curve(&aTrimmed1), &curve(&aTrimmed2)));
}

// ============================================================================
// OffsetCurve Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__OffsetCurve_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    let aLine = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aRefDir = dir(0.0, 0.0, 1.0);
    let anOffset1 =
        Handle::new(GeomOffsetCurve::new(&curve(&aLine), 5.0, aRefDir, false).unwrap());
    let anOffset2: Handle<GeomOffsetCurve> = copied(&anOffset1);

    assert_eq!(myHasher.hash(&curve(&anOffset1)), myHasher.hash(&curve(&anOffset2)));
    assert!(myHasher.equal(&curve(&anOffset1), &curve(&anOffset2)));
}

#[test]
fn GeomHash_CurveHasherTest__OffsetCurve_DifferentOffset_DifferentHash() {
    let myHasher = SetUp();
    let aLine = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aRefDir = dir(0.0, 0.0, 1.0);
    let anOffset1 =
        Handle::new(GeomOffsetCurve::new(&curve(&aLine), 5.0, aRefDir, false).unwrap());
    let anOffset2 =
        Handle::new(GeomOffsetCurve::new(&curve(&aLine), 10.0, aRefDir, false).unwrap());

    assert_ne!(myHasher.hash(&curve(&anOffset1)), myHasher.hash(&curve(&anOffset2)));
    assert!(!myHasher.equal(&curve(&anOffset1), &curve(&anOffset2)));
}

// ============================================================================
// Cross-type Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__DifferentTypes_DifferentComparison() {
    let myHasher = SetUp();
    let aLine = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let anAxis = standard_axis();
    let aCircle = circle(anAxis, 5.0);

    assert!(!myHasher.equal(&curve(&aLine), &curve(&aCircle)));
}

#[test]
fn GeomHash_CurveHasherTest__NullCurves_HandledCorrectly() {
    let myHasher = SetUp();
    let aNullCurve: Handle<dyn GeomCurve> = Handle::default();
    let aLine = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));

    assert_eq!(myHasher.hash(&aNullCurve), 0usize);
    assert!(myHasher.equal(&aNullCurve, &aNullCurve));
    assert!(!myHasher.equal(&curve(&aLine), &aNullCurve));
}

#[test]
fn GeomHash_CurveHasherTest__SameObject_Equal() {
    let myHasher = SetUp();
    let aLine = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    assert!(myHasher.equal(&curve(&aLine), &curve(&aLine)));
}

// ============================================================================
// Weighted BSpline Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__BSplineCurve_Weighted_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    let aPoles = poles(&[
        (0.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (2.0, 1.0, 0.0),
        (3.0, 0.0, 0.0),
    ]);
    let aWeights = reals(&[1.0, 2.0, 2.0, 1.0]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = integers(&[4, 4]);

    let aCurve1 = Handle::new(
        GeomBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 3, false).unwrap(),
    );
    let aCurve2: Handle<GeomBSplineCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

#[test]
fn GeomHash_CurveHasherTest__BSplineCurve_DifferentWeights_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles = poles(&[
        (0.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (2.0, 1.0, 0.0),
        (3.0, 0.0, 0.0),
    ]);
    let aWeights1 = reals(&[1.0, 2.0, 2.0, 1.0]);
    // The second weight differs.
    let aWeights2 = reals(&[1.0, 3.0, 2.0, 1.0]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = integers(&[4, 4]);

    let aCurve1 = Handle::new(
        GeomBSplineCurve::with_weights(&aPoles, &aWeights1, &aKnots, &aMults, 3, false).unwrap(),
    );
    let aCurve2 = Handle::new(
        GeomBSplineCurve::with_weights(&aPoles, &aWeights2, &aKnots, &aMults, 3, false).unwrap(),
    );

    assert!(!myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}

// ============================================================================
// Weighted Bezier Tests
// ============================================================================

#[test]
fn GeomHash_CurveHasherTest__BezierCurve_Weighted_CopiedCurves_SameHash() {
    let myHasher = SetUp();
    let aPoles = poles(&[(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, 0.0, 0.0)]);
    let aWeights = reals(&[1.0, 2.0, 1.0]);

    let aCurve1 = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aCurve2: Handle<GeomBezierCurve> = copied(&aCurve1);

    assert_eq!(myHasher.hash(&curve(&aCurve1)), myHasher.hash(&curve(&aCurve2)));
    assert!(myHasher.equal(&curve(&aCurve1), &curve(&aCurve2)));
}
