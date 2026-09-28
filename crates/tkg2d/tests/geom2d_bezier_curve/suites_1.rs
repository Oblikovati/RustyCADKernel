#[test]
fn Geom2d_BezierCurve_Test__Resolution() {
    let myOriginalCurve = SetUp();
    let aUTol: f64 = myOriginalCurve.resolution(1.0).unwrap();
    assert!(aUTol > 0.0);
}

#[test]
fn Geom2d_BezierCurve_Test__Transform() {
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
fn Geom2d_BezierCurve_Test__PolesAccess() {
    let myOriginalCurve = SetUp();
    let aPoles: Array1<'static, Point2> = myOriginalCurve.poles();
    assert_eq!(aPoles.len(), 4);
    assert!(aPoles[1].is_equal(Point2::new(0.0, 0.0), 1e-10));
}

#[test]
fn Geom2d_BezierCurve_Test__WeightsAccess_NonRational() {
    let myOriginalCurve = SetUp();
    let aWeights: Option<Array1<'static, f64>> = myOriginalCurve.weights();
    assert!(aWeights.is_none());
}

#[test]
fn Geom2d_BezierCurve_Test__MaxDegree() {
    let _myOriginalCurve = SetUp();
    assert!(Geom2dBezierCurve::max_degree() >= 25);
}

#[test]
fn Geom2d_BezierCurve_Test__RationalCurveEvaluation() {
    let _myOriginalCurve = SetUp();
    // Rational quadratic Bezier approximating circular arc
    let aPoles = poles3(
        Point2::new(1.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(0.0, 1.0),
    );
    let aWeights = weights3(1.0, 1.0 / 2.0_f64.sqrt(), 1.0);

    let aCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    assert!(aCurve.is_rational());

    let aMid: Point2 = aCurve.value(0.5).unwrap();
    let aRadius: f64 = (aMid.x() * aMid.x() + aMid.y() * aMid.y()).sqrt();
    EXPECT_NEAR(aRadius, 1.0, 1e-6);
}

#[test]
fn Geom2d_BezierCurve_Test__SetPoleWithWeight() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );
    let aWeights = weights3(1.0, 1.0, 1.0);

    let aCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    aCurve
        .set_pole_weighted(2, Point2::new(1.0, 2.0), 5.0)
        .unwrap();
    assert!(
        aCurve
            .pole(2)
            .unwrap()
            .is_equal(Point2::new(1.0, 2.0), 1e-10)
    );
    EXPECT_DOUBLE_EQ(aCurve.weight(2).unwrap(), 5.0);
    assert!(aCurve.is_rational());
}

#[test]
fn Geom2d_BezierCurve_Test__InsertPoleBeforeWithWeight() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );
    let aWeights = weights3(1.0, 2.0, 1.0);

    let aCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    aCurve
        .insert_pole_before(2, Point2::new(0.5, 0.5), 3.0)
        .unwrap();
    assert_eq!(aCurve.nb_poles(), 4);
    EXPECT_DOUBLE_EQ(aCurve.weight(2).unwrap(), 3.0);
}

#[test]
fn Geom2d_BezierCurve_Test__ClosedCurve() {
    let _myOriginalCurve = SetUp();
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 1.0);
    aPoles[3] = Point2::new(2.0, 0.0);
    aPoles[4] = Point2::new(0.0, 0.0);

    let aCurve = Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap());
    assert!(aCurve.is_closed().unwrap());
}

#[test]
fn Geom2d_BezierCurve_Test__RationalSegment() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(
        Point2::new(1.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(0.0, 1.0),
    );
    let aWeights = weights3(1.0, 1.0 / 2.0_f64.sqrt(), 1.0);

    let aCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aMid: Point2 = aCurve.value(0.5).unwrap();

    aCurve.segment(0.25, 0.75).unwrap();
    let aMidAfter: Point2 = aCurve.value(0.5).unwrap();
    assert!(aMid.is_equal(aMidAfter, 1e-6));
    assert!(aCurve.is_rational());
}

#[test]
fn Geom2d_BezierCurve_Test__RationalReverse() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );
    let aWeights = weights3(1.0, 3.0, 2.0);

    let aCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aStart: Point2 = aCurve.start_point().unwrap();
    let anEnd: Point2 = aCurve.end_point().unwrap();

    aCurve.reverse().unwrap();
    assert!(aCurve.start_point().unwrap().is_equal(anEnd, 1e-10));
    assert!(aCurve.end_point().unwrap().is_equal(aStart, 1e-10));
    EXPECT_DOUBLE_EQ(aCurve.weight(1).unwrap(), 2.0);
    EXPECT_DOUBLE_EQ(aCurve.weight(3).unwrap(), 1.0);
}

#[test]
fn Geom2d_BezierCurve_Test__Evaluation_D3() {
    let myOriginalCurve = SetUp();
    let (_aPnt, _aV1, _aV2, aV3) = myOriginalCurve.d3(0.5).unwrap();
    // D3 of degree 3 is constant
    let (_aPntB, _aV1b, _aV2b, aV3b) = myOriginalCurve.d3(0.25).unwrap();
    EXPECT_NEAR(aV3.x(), aV3b.x(), 1e-8);
    EXPECT_NEAR(aV3.y(), aV3b.y(), 1e-8);
}

#[test]
fn Geom2d_BezierCurve_Test__ReversedParameter() {
    let myOriginalCurve = SetUp();
    EXPECT_DOUBLE_EQ(myOriginalCurve.reversed_parameter(0.3).unwrap(), 0.7);
    EXPECT_DOUBLE_EQ(myOriginalCurve.reversed_parameter(0.0).unwrap(), 1.0);
}

#[test]
fn Geom2d_BezierCurve_Test__LinearCurve() {
    let _myOriginalCurve = SetUp();
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 2).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(3.0, 4.0);

    let aCurve = Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap());
    assert_eq!(aCurve.degree(), 1);

    let aMid: Point2 = aCurve.value(0.5).unwrap();
    assert!(aMid.is_equal(Point2::new(1.5, 2.0), 1e-10));
}

#[test]
fn Geom2d_BezierCurve_Test__RationalIncrease() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );
    let aWeights = weights3(1.0, 2.0, 1.0);

    let aCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aValBefore: Point2 = aCurve.value(0.5).unwrap();

    aCurve.increase(5).unwrap();
    assert_eq!(aCurve.degree(), 5);
    assert!(aCurve.is_rational());
    let aValAfter: Point2 = aCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(aValAfter, 1e-10));
}

#[test]
fn Geom2d_BezierCurve_Test__WeightsArray_NonRational_ReturnsUnitWeights() {
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
    // The reference is stable.
    assert_eq!(aAddress, myOriginalCurve.weights_array(std::ptr::from_ref));
}

#[test]
fn Geom2d_BezierCurve_Test__WeightsArray_Rational_ReturnsOwning() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );
    let aWeightsIn = weights3(1.0, 2.0, 1.0);

    let aRational = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeightsIn).unwrap());
    assert!(aRational.is_rational());

    let aAddress = aRational.weights_array(|aWeights| {
        assert_eq!(aWeights.len(), 3);
        assert!(aWeights.is_deletable());
        EXPECT_DOUBLE_EQ(aWeights[1], 1.0);
        EXPECT_DOUBLE_EQ(aWeights[2], 2.0);
        EXPECT_DOUBLE_EQ(aWeights[3], 1.0);
        std::ptr::from_ref(aWeights)
    });
    // The reference is stable.
    assert_eq!(aAddress, aRational.weights_array(std::ptr::from_ref));
}
