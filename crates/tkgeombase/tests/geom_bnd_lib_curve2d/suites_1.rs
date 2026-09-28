// =========================================================================
// BezierCurve2d
// =========================================================================

#[test]
fn GeomBndLib_Curve2dTest__BezierCurve_Simple() {
    let aPoles = array1(&[pnt2d(0.0, 0.0), pnt2d(5.0, 10.0), pnt2d(10.0, 0.0)]);

    let aBez = curve2d(Geom2dBezierCurve::new(&aPoles).unwrap());
    let aCurve = Curve2d::from_curve(&aBez).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::BezierCurve);

    let mut aBox = BoundingBox2::default();
    aCurve.add(precision::confusion(), &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax] = get2(&aBox);

    assert!(aXmin <= precision::confusion());
    assert!(aXmax >= 10.0 - precision::confusion());
    assert!(aYmin <= precision::confusion());
    assert!(aYmax >= 0.0);
}

#[test]
fn GeomBndLib_Curve2dTest__BezierCurve_CompareWithBndLib() {
    let aPoles = array1(&[
        pnt2d(0.0, 0.0),
        pnt2d(3.0, 8.0),
        pnt2d(7.0, 2.0),
        pnt2d(10.0, 5.0),
    ]);

    let aBez = curve2d(Geom2dBezierCurve::new(&aPoles).unwrap());

    let aNewBox = new_box(&aBez, precision::confusion());
    let anOldBox = old_box(&adaptor(&aBez), precision::confusion());

    // New box should be no larger than old box (may be tighter due to convex hull approach)
    let [aXminN, aXmaxN, aYminN, aYmaxN] = get2(&aNewBox);
    let [aXminO, aXmaxO, aYminO, aYmaxO] = get2(&anOldBox);
    assert!(aXminN >= aXminO - precision::confusion());
    assert!(aXmaxN <= aXmaxO + precision::confusion());
    assert!(aYminN >= aYminO - precision::confusion());
    assert!(aYmaxN <= aYmaxO + precision::confusion());
}

// =========================================================================
// BSplineCurve2d
// =========================================================================

// The degree-2 B-spline of both BSplineCurve tests.
fn quadratic_bspline() -> (Handle<dyn Geom2dCurve>, Geom2dBSplineCurve) {
    let aPoles: [Point2; 4] = [
        pnt2d(0.0, 0.0),
        pnt2d(2.0, 5.0),
        pnt2d(8.0, 5.0),
        pnt2d(10.0, 0.0),
    ];
    let aKnots = array1(&[0.0, 0.5, 1.0]);
    let aMults = array1(&[3, 1, 3]);
    let aMake = || Geom2dBSplineCurve::new(&array1(&aPoles), &aKnots, &aMults, 2, false).unwrap();
    (curve2d(aMake()), aMake())
}

#[test]
fn GeomBndLib_Curve2dTest__BSplineCurve_Simple() {
    let (aBSpl, _) = quadratic_bspline();
    let aCurve = Curve2d::from_curve(&aBSpl).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::BSplineCurve);

    let mut aBox = BoundingBox2::default();
    aCurve.add(precision::confusion(), &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax] = get2(&aBox);

    assert!(aXmin <= precision::confusion());
    assert!(aXmax >= 10.0 - precision::confusion());
    assert!(aYmin <= precision::confusion());
    assert!(aYmax >= 0.0);
}

#[test]
fn GeomBndLib_Curve2dTest__BSplineCurve_CompareWithBndLib() {
    let (aBSpl, _) = quadratic_bspline();

    let aNewBox = new_box(&aBSpl, precision::confusion());
    let anOldBox = old_box(&adaptor(&aBSpl), precision::confusion());

    CompareBoxes2d(&aNewBox, &anOldBox, precision::confusion());
}

// =========================================================================
// Adaptor constructor
// =========================================================================

#[test]
fn GeomBndLib_Curve2dTest__AdaptorConstructor_Circle() {
    let aRadius = 5.0;
    let aCircle = circle(aRadius);

    // Construct from handle
    let aBoxH = new_box(&aCircle, precision::confusion());

    // Construct from adaptor
    let anAdaptor = adaptor(&aCircle);
    let aCurveA = Curve2d::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(aCurveA.get_type(), CurveType::Circle);

    let mut aBoxA = BoundingBox2::default();
    aCurveA.add(precision::confusion(), &mut aBoxA).unwrap();

    CompareBoxes2d(&aBoxH, &aBoxA, precision::confusion());
}

// =========================================================================
// AddOptimal
// =========================================================================

#[test]
fn GeomBndLib_Curve2dTest__AddOptimal_Circle() {
    let aRadius = 5.0;
    let aCircle = circle(aRadius);
    let aCurve = Curve2d::from_curve(&aCircle).unwrap();

    let mut aBoxOpt = BoundingBox2::default();
    aCurve.add_optimal(0.0, &mut aBoxOpt).unwrap();

    let [aXminO, aXmaxO, aYminO, aYmaxO] = get2(&aBoxOpt);

    // Analytical circle should give precise box
    EXPECT_NEAR(aXminO, -aRadius, precision::confusion());
    EXPECT_NEAR(aYminO, -aRadius, precision::confusion());
    EXPECT_NEAR(aXmaxO, aRadius, precision::confusion());
    EXPECT_NEAR(aYmaxO, aRadius, precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__AdaptorConstructor_TrimmedRangeUsedForFullAdd() {
    let aCircle = circle(10.0);
    let anAdaptor = Geom2dAdaptorCurve::with_range(&aCircle, 0.0, PI / 2.0).unwrap();

    let mut aNewBox = BoundingBox2::default();
    Curve2d::from_adaptor(&anAdaptor)
        .unwrap()
        .add(precision::confusion(), &mut aNewBox)
        .unwrap();

    let anOldBox = old_box(&anAdaptor, precision::confusion());

    CompareBoxes2d(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__TrimmedCurveHandle_Fallback_CompareWithBndLib() {
    let aCircle = circle(10.0);
    let aTrim = curve2d(Geom2dTrimmedCurve::new(&aCircle, 0.0, PI / 2.0, true, true).unwrap());

    let aNewBox = new_box(&aTrim, precision::confusion());
    let anOldBox = old_box(&adaptor(&aTrim), precision::confusion());

    assert!(!aNewBox.is_void());
    ExpectConservativeEnvelope2d(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__TrimmedCurveHandle_UsesBasisSpecialization() {
    let aCircle = circle(10.0);
    let aTrim = curve2d(Geom2dTrimmedCurve::new(&aCircle, 0.0, PI / 2.0, true, true).unwrap());
    let aCurve = Curve2d::from_curve(&aTrim).unwrap();
    assert_eq!(aCurve.get_type(), CurveType::Circle);

    let mut aNewBox = BoundingBox2::default();
    aCurve.add(precision::confusion(), &mut aNewBox).unwrap();
    let anOldBox = old_box(&adaptor(&aTrim), precision::confusion());
    CompareBoxes2d(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_Curve2dTest__SamplingHelpers2d_MatchesLegacyPolicy() {
    let aCircleGA = adaptor(&circle(10.0));
    assert_eq!(
        sampling::compute_nb_samples_2d(&aCircleGA, 0.0, 1.0).unwrap(),
        17
    );

    let aDegree = 3;
    let aNbKnots = 40;
    let aNbPoles = 42;
    let aPoles: Vec<Point2> = (1..=aNbPoles)
        .map(|i| {
            let t = f64::from(i - 1) / f64::from(aNbPoles - 1);
            pnt2d(10.0 * t, t)
        })
        .collect();
    let aKnots: Vec<f64> = (1..=aNbKnots).map(|i| f64::from(i - 1)).collect();
    let mut aMults: Vec<i32> = vec![1; aNbKnots as usize];
    aMults[0] = aDegree + 1;
    aMults[(aNbKnots - 1) as usize] = aDegree + 1;

    let aBSpl = Geom2dBSplineCurve::new(
        &array1(&aPoles),
        &array1(&aKnots),
        &array1(&aMults),
        aDegree,
        false,
    )
    .unwrap();
    let (aFirst, aLast) = (aBSpl.first_parameter(), aBSpl.last_parameter());
    let aBSplGA = adaptor(&curve2d(aBSpl));
    assert_eq!(
        sampling::compute_nb_samples_2d(&aBSplGA, aFirst, aLast).unwrap(),
        23
    );
}
