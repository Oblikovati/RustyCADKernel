// =========================================================================
// Line
// =========================================================================

#[test]
fn GeomBndLib_CurveTest__Line_FiniteSegment() {
    let aLine = curve(GeomLine::from_point(pnt(0.0, 0.0, 0.0), dir(1.0, 1.0, 0.0)));
    let aCurve = Curve::from_curve(&aLine).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Line);

    let mut aBox = BoundingBox3::default();
    aCurve.add_range(0.0, 10.0, 0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    // Line from (0,0,0) in direction (1,1,0) normalized, parameter 10
    // Endpoint: (10/sqrt(2), 10/sqrt(2), 0)
    let aLen = 10.0 / 2.0_f64.sqrt();
    EXPECT_NEAR(aXmin, 0.0, precision::confusion());
    EXPECT_NEAR(aYmin, 0.0, precision::confusion());
    EXPECT_NEAR(aZmin, 0.0, precision::confusion());
    EXPECT_NEAR(aXmax, aLen, precision::confusion());
    EXPECT_NEAR(aYmax, aLen, precision::confusion());
    EXPECT_NEAR(aZmax, 0.0, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__Line_CompareWithBndLib() {
    let aLine = curve(GeomLine::from_point(pnt(1.0, 2.0, 3.0), dir(0.0, 0.0, 1.0)));

    let aNewBox = new_box_range(&aLine, 0.0, 5.0, precision::confusion());
    let anOldBox = old_box_range(&aLine, 0.0, 5.0, precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Circle
// =========================================================================

#[test]
fn GeomBndLib_CurveTest__Circle_Full() {
    let aRadius = 5.0;
    let aCircle = circle(aRadius);
    let aCurve = Curve::from_curve(&aCircle).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Circle);

    let mut aBox = BoundingBox3::default();
    aCurve.add(0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    EXPECT_NEAR(aXmin, -aRadius, precision::confusion());
    EXPECT_NEAR(aYmin, -aRadius, precision::confusion());
    EXPECT_NEAR(aXmax, aRadius, precision::confusion());
    EXPECT_NEAR(aYmax, aRadius, precision::confusion());
    EXPECT_NEAR(aZmin, 0.0, precision::confusion());
    EXPECT_NEAR(aZmax, 0.0, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__Circle_Arc() {
    let aRadius = 10.0;
    let aCircle = circle(aRadius);
    let aCurve = Curve::from_curve(&aCircle).unwrap();

    let mut aBox = BoundingBox3::default();
    // Pass 0 tolerance for precise reference comparison
    aCurve.add_range(0.0, PI / 2.0, 0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    EXPECT_NEAR(aXmin, 0.0, precision::confusion());
    EXPECT_NEAR(aYmin, 0.0, precision::confusion());
    EXPECT_NEAR(aXmax, aRadius, precision::confusion());
    EXPECT_NEAR(aYmax, aRadius, precision::confusion());
    EXPECT_NEAR(aZmin, 0.0, precision::confusion());
    EXPECT_NEAR(aZmax, 0.0, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__Circle_CompareWithBndLib() {
    let aCircle = circle(7.5);

    let aNewBox = new_box(&aCircle, precision::confusion());
    let anOldBox = old_box(&adaptor(&aCircle), precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__Circle_Arc_CompareWithBndLib() {
    let aCircle = circle(7.5);

    let aNewBox = new_box_range(&aCircle, 0.5, 2.5, precision::confusion());
    let anOldBox = old_box_range(&aCircle, 0.5, 2.5, precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Ellipse
// =========================================================================

#[test]
fn GeomBndLib_CurveTest__Ellipse_Full() {
    let aMajR = 10.0;
    let aMinR = 5.0;
    let anEllipse = ellipse(aMajR, aMinR);
    let aCurve = Curve::from_curve(&anEllipse).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Ellipse);

    let mut aBox = BoundingBox3::default();
    aCurve.add(0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    EXPECT_NEAR(aXmin, -aMajR, precision::confusion());
    EXPECT_NEAR(aXmax, aMajR, precision::confusion());
    EXPECT_NEAR(aYmin, -aMinR, precision::confusion());
    EXPECT_NEAR(aYmax, aMinR, precision::confusion());
    EXPECT_NEAR(aZmin, 0.0, precision::confusion());
    EXPECT_NEAR(aZmax, 0.0, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__Ellipse_CompareWithBndLib() {
    let anEllipse = ellipse(8.0, 3.0);

    let aNewBox = new_box(&anEllipse, precision::confusion());
    let anOldBox = old_box(&adaptor(&anEllipse), precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__Ellipse_Arc_CompareWithBndLib() {
    let anEllipse = ellipse(8.0, 3.0);

    let aNewBox = new_box_range(&anEllipse, 0.3, 2.0, precision::confusion());
    let anOldBox = old_box_range(&anEllipse, 0.3, 2.0, precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Hyperbola
// =========================================================================

#[test]
fn GeomBndLib_CurveTest__Hyperbola_FiniteArc() {
    let aMajR = 5.0;
    let aMinR = 3.0;
    let aHyp = curve(GeomHyperbola::new(gp::xoy(), aMajR, aMinR).unwrap());
    let aCurve = Curve::from_curve(&aHyp).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Hyperbola);

    let mut aBox = BoundingBox3::default();
    let aU = 1.0;
    aCurve.add_range(-aU, aU, 0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, _aZmin, _aZmax] = get3(&aBox);

    // Hyperbola: x = aMajR * cosh(t), y = aMinR * sinh(t)
    let aXEnd = aMajR * f64::cosh(aU);
    let aYEnd = aMinR * f64::sinh(aU);

    // Box should contain the vertex (aMajR, 0) and endpoints
    assert!(aXmin <= aMajR + precision::confusion());
    assert!(aXmax >= aXEnd - precision::confusion());
    assert!(aYmin <= -aYEnd + precision::confusion());
    assert!(aYmax >= aYEnd - precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__Hyperbola_CompareWithBndLib() {
    let aHyp = curve(GeomHyperbola::new(gp::xoy(), 5.0, 3.0).unwrap());

    let aNewBox = new_box_range(&aHyp, -1.0, 1.0, precision::confusion());
    let anOldBox = old_box_range(&aHyp, -1.0, 1.0, precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Parabola
// =========================================================================

#[test]
fn GeomBndLib_CurveTest__Parabola_FiniteArc() {
    let aFocal = 2.0;
    let aParab = curve(GeomParabola::new(gp::xoy(), aFocal).unwrap());
    let aCurve = Curve::from_curve(&aParab).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Parabola);

    let mut aBox = BoundingBox3::default();
    let aU = 5.0;
    aCurve.add_range(-aU, aU, 0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, _aZmin, _aZmax] = get3(&aBox);

    // Parabola: x = t^2/(4*focal), y = t
    let aXEnd = aU * aU / (4.0 * aFocal);

    assert!(aXmin <= precision::confusion());
    assert!(aXmax >= aXEnd - precision::confusion());
    assert!(aYmin <= -aU + precision::confusion());
    assert!(aYmax >= aU - precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__Parabola_CompareWithBndLib() {
    let aParab = curve(GeomParabola::new(gp::xoy(), 2.0).unwrap());

    let aNewBox = new_box_range(&aParab, -5.0, 5.0, precision::confusion());
    let anOldBox = old_box_range(&aParab, -5.0, 5.0, precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}
