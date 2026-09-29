// =========================================================================
// Line2d
// =========================================================================

#[test]
fn GeomBndLib_Curve2dTest__Line_FiniteSegment() {
    let aLine = curve2d(Geom2dLine::from_point(pnt2d(0.0, 0.0), dir2d(1.0, 1.0)));
    let aCurve = Curve2d::from_curve(&aLine).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Line);

    let mut aBox = BoundingBox2::default();
    aCurve.add_range(0.0, 10.0, 0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax] = get2(&aBox);

    // Line from (0,0) in direction (1,1) normalized, parameter 10
    let aLen = 10.0 / 2.0_f64.sqrt();
    EXPECT_NEAR(aXmin, 0.0, precision::confusion());
    EXPECT_NEAR(aYmin, 0.0, precision::confusion());
    EXPECT_NEAR(aXmax, aLen, precision::confusion());
    EXPECT_NEAR(aYmax, aLen, precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__Line_CompareWithBndLib() {
    let aLine = curve2d(Geom2dLine::from_point(pnt2d(1.0, 2.0), dir2d(0.0, 1.0)));

    let aNewBox = new_box_range(&aLine, 0.0, 5.0, precision::confusion());
    let anOldBox = old_box_range(&aLine, 0.0, 5.0, precision::confusion());

    CompareBoxes2d(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Circle2d
// =========================================================================

#[test]
fn GeomBndLib_Curve2dTest__Circle_Full() {
    let aRadius = 5.0;
    let aCircle = circle(aRadius);
    let aCurve = Curve2d::from_curve(&aCircle).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Circle);

    let mut aBox = BoundingBox2::default();
    aCurve.add(0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax] = get2(&aBox);

    EXPECT_NEAR(aXmin, -aRadius, precision::confusion());
    EXPECT_NEAR(aYmin, -aRadius, precision::confusion());
    EXPECT_NEAR(aXmax, aRadius, precision::confusion());
    EXPECT_NEAR(aYmax, aRadius, precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__Circle_Arc() {
    let aRadius = 10.0;
    let aCircle = circle(aRadius);
    let aCurve = Curve2d::from_curve(&aCircle).unwrap();

    let mut aBox = BoundingBox2::default();
    // Pass 0 tolerance for precise reference comparison
    aCurve.add_range(0.0, PI / 2.0, 0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax] = get2(&aBox);

    EXPECT_NEAR(aXmin, 0.0, precision::confusion());
    EXPECT_NEAR(aYmin, 0.0, precision::confusion());
    EXPECT_NEAR(aXmax, aRadius, precision::confusion());
    EXPECT_NEAR(aYmax, aRadius, precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__Circle_CompareWithBndLib() {
    let aCircle = circle(7.5);

    let aNewBox = new_box(&aCircle, precision::confusion());
    let anOldBox = old_box(&adaptor(&aCircle), precision::confusion());

    CompareBoxes2d(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Ellipse2d
// =========================================================================

#[test]
fn GeomBndLib_Curve2dTest__Ellipse_Full() {
    let aMajR = 10.0;
    let aMinR = 5.0;
    let anEllipse = curve2d(Geom2dEllipse::new(frame(), aMajR, aMinR).unwrap());
    let aCurve = Curve2d::from_curve(&anEllipse).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Ellipse);

    let mut aBox = BoundingBox2::default();
    aCurve.add(0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax] = get2(&aBox);

    EXPECT_NEAR(aXmin, -aMajR, precision::confusion());
    EXPECT_NEAR(aXmax, aMajR, precision::confusion());
    EXPECT_NEAR(aYmin, -aMinR, precision::confusion());
    EXPECT_NEAR(aYmax, aMinR, precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__Ellipse_CompareWithBndLib() {
    let anEllipse = curve2d(Geom2dEllipse::new(frame(), 8.0, 3.0).unwrap());

    let aNewBox = new_box(&anEllipse, precision::confusion());
    let anOldBox = old_box(&adaptor(&anEllipse), precision::confusion());

    CompareBoxes2d(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Hyperbola2d
// =========================================================================

#[test]
fn GeomBndLib_Curve2dTest__Hyperbola_FiniteArc() {
    let aMajR = 5.0;
    let aMinR = 3.0;
    let aHyp = curve2d(Geom2dHyperbola::new(frame(), aMajR, aMinR).unwrap());
    let aCurve = Curve2d::from_curve(&aHyp).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Hyperbola);

    let mut aBox = BoundingBox2::default();
    let aU = 1.0;
    aCurve.add_range(-aU, aU, 0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax] = get2(&aBox);

    let aXEnd = aMajR * f64::cosh(aU);
    let aYEnd = aMinR * f64::sinh(aU);

    assert!(aXmin <= aMajR + precision::confusion());
    assert!(aXmax >= aXEnd - precision::confusion());
    assert!(aYmin <= -aYEnd + precision::confusion());
    assert!(aYmax >= aYEnd - precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__Hyperbola_CompareWithBndLib() {
    let aHyp = curve2d(Geom2dHyperbola::new(frame(), 5.0, 3.0).unwrap());

    let aNewBox = new_box_range(&aHyp, -1.0, 1.0, precision::confusion());
    let anOldBox = old_box_range(&aHyp, -1.0, 1.0, precision::confusion());

    CompareBoxes2d(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Parabola2d
// =========================================================================

#[test]
fn GeomBndLib_Curve2dTest__Parabola_FiniteArc() {
    let aFocal = 2.0;
    let aParab = curve2d(Geom2dParabola::new(frame(), aFocal).unwrap());
    let aCurve = Curve2d::from_curve(&aParab).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Parabola);

    let mut aBox = BoundingBox2::default();
    let aU = 5.0;
    aCurve.add_range(-aU, aU, 0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax] = get2(&aBox);

    let aXEnd = aU * aU / (4.0 * aFocal);

    assert!(aXmin <= precision::confusion());
    assert!(aXmax >= aXEnd - precision::confusion());
    assert!(aYmin <= -aU + precision::confusion());
    assert!(aYmax >= aU - precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__Parabola_CompareWithBndLib() {
    let aParab = curve2d(Geom2dParabola::new(frame(), 2.0).unwrap());

    let aNewBox = new_box_range(&aParab, -5.0, 5.0, precision::confusion());
    let anOldBox = old_box_range(&aParab, -5.0, 5.0, precision::confusion());

    CompareBoxes2d(&aNewBox, &anOldBox, precision::confusion());
}
