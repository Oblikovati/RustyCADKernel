#[test]
fn Geom2d_BSplineCurve_Test__Resolution() {
    let myOriginalCurve = SetUp();
    let aUTol: f64 = myOriginalCurve.resolution(1.0).unwrap();
    assert!(aUTol > 0.0);
}

#[test]
fn Geom2d_BSplineCurve_Test__Transform() {
    let myOriginalCurve = SetUp();
    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(10.0, 20.0));
    let aPtBefore: Point2 = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve.transform(aTrsf).unwrap();
    let aPtAfter: Point2 = myOriginalCurve.value(0.5).unwrap();
    EXPECT_NEAR(aPtAfter.x(), aPtBefore.x() + 10.0, 1e-10);
    EXPECT_NEAR(aPtAfter.y(), aPtBefore.y() + 20.0, 1e-10);
}

#[test]
fn Geom2d_BSplineCurve_Test__PeriodicCurve() {
    let _myOriginalCurve = SetUp();
    // Periodic BSpline: degree 3, 6 knots of mult 1 -> NbPoles = 1+4 = 5
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 5).unwrap();
    aPoles[1] = Point2::new(1.0, 0.0);
    aPoles[2] = Point2::new(0.309, 0.951);
    aPoles[3] = Point2::new(-0.809, 0.588);
    aPoles[4] = Point2::new(-0.809, -0.588);
    aPoles[5] = Point2::new(0.309, -0.951);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 6).unwrap();
    for i in 1..=6 {
        aKnots[i] = f64::from(i - 1) * 0.2;
    }

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 6).unwrap();
    aMults.init(1);

    let aCurve = Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, true).unwrap());
    assert!(aCurve.is_periodic());

    let aVal1: Point2 = aCurve.value(0.5).unwrap();
    aCurve.set_not_periodic().unwrap();
    assert!(!aCurve.is_periodic());

    let aVal2: Point2 = aCurve.value(0.5).unwrap();
    assert!(aVal1.is_equal(aVal2, 1e-10));
}

#[test]
fn Geom2d_BSplineCurve_Test__WeightsAccess_NonRational() {
    let myOriginalCurve = SetUp();
    let aWeights: Option<Array1<'static, f64>> = myOriginalCurve.weights();
    assert!(aWeights.is_none());
}

#[test]
fn Geom2d_BSplineCurve_Test__KnotAccess() {
    let myOriginalCurve = SetUp();
    EXPECT_DOUBLE_EQ(myOriginalCurve.knot(1).unwrap(), 0.0);
    EXPECT_DOUBLE_EQ(myOriginalCurve.knot(2).unwrap(), 1.0);

    let aKnots: Array1<'static, f64> = myOriginalCurve.knots();
    assert_eq!(aKnots.len(), 2);

    let aMults: Array1<'static, i32> = myOriginalCurve.multiplicities();
    assert_eq!(aMults.len(), 2);
    assert_eq!(aMults[1], 4);
}

#[test]
fn Geom2d_BSplineCurve_Test__CopyIndependence() {
    let myOriginalCurve = SetUp();
    let aCopy = Handle::new(Geom2dBSplineCurve::copy_of(&myOriginalCurve).unwrap());
    myOriginalCurve
        .set_pole(2, Point2::new(10.0, 10.0))
        .unwrap();
    assert!(
        !aCopy
            .pole(2)
            .unwrap()
            .is_equal(Point2::new(10.0, 10.0), 1e-10)
    );
}

#[test]
fn Geom2d_BSplineCurve_Test__SetWeight() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );
    let aWeights = weights3(1.0, 1.0, 1.0);
    let (aKnots, aMults) = end_knots(3);

    let aCurve = Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 2, false).unwrap(),
    );

    let aMidBefore: Point2 = aCurve.value(0.5).unwrap();
    aCurve.set_weight(2, 5.0).unwrap();
    EXPECT_DOUBLE_EQ(aCurve.weight(2).unwrap(), 5.0);
    assert!(aCurve.is_rational());

    let aMidAfter: Point2 = aCurve.value(0.5).unwrap();
    assert!(aMidAfter.y() > aMidBefore.y());
}

#[test]
fn Geom2d_BSplineCurve_Test__MovePoint() {
    let myOriginalCurve = SetUp();
    let aTarget = Point2::new(1.5, 0.8);
    let (aFirst, _aLast) = myOriginalCurve
        .move_point(0.5, aTarget, 1, myOriginalCurve.nb_poles())
        .unwrap();
    assert!(aFirst > 0);
    let aMoved: Point2 = myOriginalCurve.value(0.5).unwrap();
    assert!(aMoved.is_equal(aTarget, 1e-6));
}

#[test]
fn Geom2d_BSplineCurve_Test__LocateU() {
    let myOriginalCurve = SetUp();
    myOriginalCurve.insert_knot(0.5, 1, 0.0).unwrap();
    let (anI1, anI2) = myOriginalCurve.locate_u(0.25, 1e-10, false).unwrap();
    assert!(anI1 >= 1);
    assert!(anI2 <= myOriginalCurve.nb_knots());
}

#[test]
fn Geom2d_BSplineCurve_Test__LocalD1() {
    let myOriginalCurve = SetUp();
    myOriginalCurve.insert_knot(0.5, 1, 0.0).unwrap();
    let (aPnt, aV1) = myOriginalCurve.d1(0.25).unwrap();
    let (aPntL, aV1L) = myOriginalCurve.local_d1(0.25, 1, 2).unwrap();
    assert!(aPnt.is_equal(aPntL, 1e-10));
    EXPECT_NEAR(aV1.x(), aV1L.x(), 1e-10);
    EXPECT_NEAR(aV1.y(), aV1L.y(), 1e-10);
}

#[test]
fn Geom2d_BSplineCurve_Test__RationalCurveSegment() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(
        Point2::new(1.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(0.0, 1.0),
    );
    let aWeights = weights3(1.0, 1.0 / 2.0_f64.sqrt(), 1.0);
    let (aKnots, aMults) = end_knots(3);

    let aCurve = Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 2, false).unwrap(),
    );

    let aMid: Point2 = aCurve.value(0.5).unwrap();
    aCurve
        .segment(0.25, 0.75, precision::p_confusion())
        .unwrap();
    let aMidAfter: Point2 = aCurve.value(0.5).unwrap();
    assert!(aMid.is_equal(aMidAfter, 1e-6));
    assert!(aCurve.is_rational());
}

#[test]
fn Geom2d_BSplineCurve_Test__ClosedCurve() {
    let _myOriginalCurve = SetUp();
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 1.0);
    aPoles[3] = Point2::new(2.0, 0.0);
    aPoles[4] = Point2::new(0.0, 0.0);
    let (aKnots, aMults) = end_knots(4);

    let aCurve = Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap());
    assert!(aCurve.is_closed().unwrap());
}

#[test]
fn Geom2d_BSplineCurve_Test__InsertKnots_Multiple() {
    let myOriginalCurve = SetUp();
    let mut aNewKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aNewKnots[1] = 0.25;
    aNewKnots[2] = 0.75;

    let mut aNewMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aNewMults[1] = 1;
    aNewMults[2] = 1;

    let aValBefore: Point2 = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve
        .insert_knots(&aNewKnots, &aNewMults, 0.0, false)
        .unwrap();
    assert_eq!(myOriginalCurve.nb_knots(), 4);
    let aValAfter: Point2 = myOriginalCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(aValAfter, 1e-10));
}

#[test]
fn Geom2d_BSplineCurve_Test__WeightsArray_NonRational_ReturnsUnitWeights() {
    let myOriginalCurve = SetUp();
    assert!(!myOriginalCurve.is_rational());

    let aAddress = myOriginalCurve.weights_array(|aWeights| {
        assert_eq!(aWeights.len() as i32, myOriginalCurve.nb_poles());
        assert!(!aWeights.is_deletable());
        for i in 1..=aWeights.len() as i32 {
            EXPECT_DOUBLE_EQ(aWeights[i], 1.0);
        }
        std::ptr::from_ref(aWeights)
    });
    // Verify that the reference is stable
    assert_eq!(aAddress, myOriginalCurve.weights_array(std::ptr::from_ref));
}

#[test]
fn Geom2d_BSplineCurve_Test__WeightsArray_Rational_ReturnsOwning() {
    let _myOriginalCurve = SetUp();
    // Create a rational BSpline
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 1.0);
    aPoles[3] = Point2::new(2.0, 1.0);
    aPoles[4] = Point2::new(3.0, 0.0);

    let mut aWeightsIn: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    aWeightsIn[1] = 1.0;
    aWeightsIn[2] = 2.0;
    aWeightsIn[3] = 3.0;
    aWeightsIn[4] = 1.0;
    let (aKnots, aMults) = end_knots(4);

    let aRational = Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeightsIn, &aKnots, &aMults, 3, false).unwrap(),
    );
    assert!(aRational.is_rational());

    let aAddress = aRational.weights_array(|aWeights| {
        assert_eq!(aWeights.len(), 4);
        assert!(aWeights.is_deletable());
        EXPECT_DOUBLE_EQ(aWeights[1], 1.0);
        EXPECT_DOUBLE_EQ(aWeights[2], 2.0);
        EXPECT_DOUBLE_EQ(aWeights[3], 3.0);
        EXPECT_DOUBLE_EQ(aWeights[4], 1.0);
        std::ptr::from_ref(aWeights)
    });
    // Verify that the reference is stable
    assert_eq!(aAddress, aRational.weights_array(std::ptr::from_ref));
}
