// =========================================================================
// BezierCurve
// =========================================================================

#[test]
fn GeomBndLib_CurveTest__BezierCurve_Simple() {
    let aPoles = array1(&[pnt(0.0, 0.0, 0.0), pnt(5.0, 10.0, 0.0), pnt(10.0, 0.0, 0.0)]);

    let aBez = curve(GeomBezierCurve::new(&aPoles).unwrap());
    let aCurve = Curve::from_curve(&aBez).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::BezierCurve);

    let mut aBox = BoundingBox3::default();
    aCurve.add(precision::confusion(), &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    // Bezier convex hull: x in [0,10], y in [0, <=10], z = 0
    assert!(aXmin <= precision::confusion());
    assert!(aXmax >= 10.0 - precision::confusion());
    assert!(aYmin <= precision::confusion());
    assert!(aYmax >= 0.0);
    EXPECT_NEAR(aZmin, 0.0, precision::confusion());
    EXPECT_NEAR(aZmax, 0.0, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__BezierCurve_CompareWithBndLib() {
    let aPoles = array1(&[
        pnt(0.0, 0.0, 0.0),
        pnt(3.0, 8.0, 1.0),
        pnt(7.0, 2.0, -1.0),
        pnt(10.0, 5.0, 0.0),
    ]);

    let aBez = curve(GeomBezierCurve::new(&aPoles).unwrap());

    let aNewBox = new_box(&aBez, precision::confusion());
    let anOldBox = old_box(&adaptor(&aBez), precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// BSplineCurve
// =========================================================================

// The degree-2 B-spline of the two BSplineCurve tests with the given poles.
fn quadratic_bspline(thePoles: &[rustycad_tkmath::gp::Point3]) -> Handle<dyn GeomCurve> {
    let aKnots = array1(&[0.0, 0.5, 1.0]);
    let aMults = array1(&[3, 1, 3]);
    curve(GeomBSplineCurve::new(&array1(thePoles), &aKnots, &aMults, 2, false).unwrap())
}

#[test]
fn GeomBndLib_CurveTest__BSplineCurve_Simple() {
    let aBSpl = quadratic_bspline(&[
        pnt(0.0, 0.0, 0.0),
        pnt(2.0, 5.0, 0.0),
        pnt(8.0, 5.0, 0.0),
        pnt(10.0, 0.0, 0.0),
    ]);
    let aCurve = Curve::from_curve(&aBSpl).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::BSplineCurve);

    let mut aBox = BoundingBox3::default();
    aCurve.add(precision::confusion(), &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    assert!(aXmin <= precision::confusion());
    assert!(aXmax >= 10.0 - precision::confusion());
    assert!(aYmin <= precision::confusion());
    assert!(aYmax >= 0.0);
    EXPECT_NEAR(aZmin, 0.0, precision::confusion());
    EXPECT_NEAR(aZmax, 0.0, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__BSplineCurve_CompareWithBndLib() {
    let aBSpl = quadratic_bspline(&[
        pnt(0.0, 0.0, 0.0),
        pnt(2.0, 5.0, 1.0),
        pnt(8.0, 5.0, -1.0),
        pnt(10.0, 0.0, 0.0),
    ]);

    let aNewBox = new_box(&aBSpl, precision::confusion());
    let anOldBox = old_box(&adaptor(&aBSpl), precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__BSplineCurve_TrimmedRange_CompareWithBndLib() {
    let aPoles = array1(&[
        pnt(0.0, 0.0, 0.0),
        pnt(10.0, 20.0, 0.0),
        pnt(20.0, -10.0, 5.0),
        pnt(30.0, 40.0, 0.0),
        pnt(40.0, 5.0, -5.0),
        pnt(50.0, 15.0, 0.0),
        pnt(60.0, 0.0, 0.0),
    ]);
    let aKnots = array1(&[0.0, 1.0, 2.0, 3.0, 4.0]);
    let aMults = array1(&[4, 1, 1, 1, 4]);

    let aBSpl = curve(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap());

    let aU1 = 1.82;
    let aU2 = 2.04;

    let aNewBox = new_box_range(&aBSpl, aU1, aU2, precision::confusion());
    let anOldBox = old_box_range(&aBSpl, aU1, aU2, precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Adaptor constructor
// =========================================================================

#[test]
fn GeomBndLib_CurveTest__AdaptorConstructor_Circle() {
    let aRadius = 5.0;
    let aCircle = circle(aRadius);

    // Construct from handle
    let aBoxH = new_box(&aCircle, precision::confusion());

    // Construct from adaptor
    let anAdaptor = adaptor(&aCircle);
    let aCurveA = Curve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(aCurveA.get_type(), CurveType::Circle);

    let mut aBoxA = BoundingBox3::default();
    aCurveA.add(precision::confusion(), &mut aBoxA).unwrap();

    CompareBoxes(&aBoxH, &aBoxA, precision::confusion());
}

// =========================================================================
// AddOptimal
// =========================================================================

#[test]
fn GeomBndLib_CurveTest__AddOptimal_Circle() {
    let aRadius = 5.0;
    let aCircle = circle(aRadius);
    let aCurve = Curve::from_curve(&aCircle).unwrap();

    let mut aBox = BoundingBox3::default();
    aCurve.add(precision::confusion(), &mut aBox).unwrap();

    let mut aBoxOpt = BoundingBox3::default();
    aCurve
        .add_optimal(precision::confusion(), &mut aBoxOpt)
        .unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);
    let [aXminO, aXmaxO, aYminO, aYmaxO, aZminO, aZmaxO] = get3(&aBoxOpt);

    // AddOptimal should produce a box no larger than Add
    assert!(aXminO >= aXmin - precision::confusion());
    assert!(aYminO >= aYmin - precision::confusion());
    assert!(aZminO >= aZmin - precision::confusion());
    assert!(aXmaxO <= aXmax + precision::confusion());
    assert!(aYmaxO <= aYmax + precision::confusion());
    assert!(aZmaxO <= aZmax + precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__AddOptimal_CompareWithBndLib() {
    let anEllipse = ellipse(8.0, 3.0);
    let anAdaptor = adaptor(&anEllipse);

    let mut aNewBox = BoundingBox3::default();
    Curve::from_curve(&anEllipse)
        .unwrap()
        .add_optimal(precision::confusion(), &mut aNewBox)
        .unwrap();

    let mut anOldBox = BoundingBox3::default();
    Add3dCurve::add_optimal(&anAdaptor, precision::confusion(), &mut anOldBox).unwrap();

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__AdaptorConstructor_TrimmedRangeUsedForFullAdd() {
    let aCircle = circle(10.0);
    let anAdaptor = GeomAdaptorCurve::with_range(&aCircle, 0.0, PI / 2.0).unwrap();

    let mut aNewBox = BoundingBox3::default();
    Curve::from_adaptor(&anAdaptor)
        .unwrap()
        .add(precision::confusion(), &mut aNewBox)
        .unwrap();

    let anOldBox = old_box(&anAdaptor, precision::confusion());

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__TrimmedCurveHandle_Fallback_CompareWithBndLib() {
    let aCircle = circle(10.0);
    let aTrim = curve(GeomTrimmedCurve::new(&aCircle, 0.0, PI / 2.0, true, true).unwrap());

    let aNewBox = new_box(&aTrim, precision::confusion());
    let anOldBox = old_box(&adaptor(&aTrim), precision::confusion());

    assert!(!aNewBox.is_void());
    ExpectConservativeEnvelope(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__TrimmedCurveHandle_UsesBasisSpecialization() {
    let aCircle = circle(10.0);
    let aTrim = curve(GeomTrimmedCurve::new(&aCircle, 0.0, PI / 2.0, true, true).unwrap());
    let aCurve = Curve::from_curve(&aTrim).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Circle);

    let mut aNewBox = BoundingBox3::default();
    aCurve.add(precision::confusion(), &mut aNewBox).unwrap();
    let anOldBox = old_box(&adaptor(&aTrim), precision::confusion());
    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_CurveTest__SamplingHelpers3d_MatchesLegacyPolicy() {
    let aCircleGA = adaptor(&circle(10.0));
    assert_eq!(
        sampling::compute_nb_samples(&aCircleGA, 0.0, 1.0).unwrap(),
        33
    );

    let aDegree = 3;
    let aNbKnots = 40;
    let aNbPoles = 42;
    let aPoles: Vec<_> = (1..=aNbPoles)
        .map(|i| {
            let t = f64::from(i - 1) / f64::from(aNbPoles - 1);
            pnt(10.0 * t, t, 0.0)
        })
        .collect();
    let aKnots: Vec<f64> = (1..=aNbKnots).map(|i| f64::from(i - 1)).collect();
    let mut aMults: Vec<i32> = vec![1; aNbKnots as usize];
    aMults[0] = aDegree + 1;
    aMults[(aNbKnots - 1) as usize] = aDegree + 1;

    let aBSpl = GeomBSplineCurve::new(
        &array1(&aPoles),
        &array1(&aKnots),
        &array1(&aMults),
        aDegree,
        false,
    )
    .unwrap();
    let (aFirst, aLast) = (aBSpl.first_parameter(), aBSpl.last_parameter());
    let aBSplGA = adaptor(&curve(aBSpl));
    assert_eq!(
        sampling::compute_nb_samples(&aBSplGA, aFirst, aLast).unwrap(),
        312
    );
}
