#[test]
fn Geom2d_BSplineCurve_Test__CopyConstructorBasicProperties() {
    let myOriginalCurve = SetUp();
    // Test copy constructor
    let aCopiedCurve = Handle::new(Geom2dBSplineCurve::copy_of(&myOriginalCurve).unwrap());

    // Verify basic properties are identical
    assert_eq!(myOriginalCurve.degree(), aCopiedCurve.degree());
    assert_eq!(myOriginalCurve.nb_poles(), aCopiedCurve.nb_poles());
    assert_eq!(myOriginalCurve.nb_knots(), aCopiedCurve.nb_knots());
    assert_eq!(myOriginalCurve.is_periodic(), aCopiedCurve.is_periodic());
    assert_eq!(myOriginalCurve.is_rational(), aCopiedCurve.is_rational());
}

#[test]
fn Geom2d_BSplineCurve_Test__CopyConstructorPoles() {
    let myOriginalCurve = SetUp();
    let aCopiedCurve = Handle::new(Geom2dBSplineCurve::copy_of(&myOriginalCurve).unwrap());

    // Verify all poles are identical
    for i in 1..=myOriginalCurve.nb_poles() {
        let anOrigPole: Point2 = myOriginalCurve.pole(i).unwrap();
        let aCopyPole: Point2 = aCopiedCurve.pole(i).unwrap();
        assert!(anOrigPole.is_equal(aCopyPole, 1e-10));
    }
}

#[test]
fn Geom2d_BSplineCurve_Test__CopyConstructorKnots() {
    let myOriginalCurve = SetUp();
    let aCopiedCurve = Handle::new(Geom2dBSplineCurve::copy_of(&myOriginalCurve).unwrap());

    // Verify all knots are identical
    for i in 1..=myOriginalCurve.nb_knots() {
        EXPECT_DOUBLE_EQ(
            myOriginalCurve.knot(i).unwrap(),
            aCopiedCurve.knot(i).unwrap(),
        );
        assert_eq!(
            myOriginalCurve.multiplicity(i).unwrap(),
            aCopiedCurve.multiplicity(i).unwrap()
        );
    }
}

#[test]
fn Geom2d_BSplineCurve_Test__CopyMethodUsesOptimizedConstructor() {
    let myOriginalCurve = SetUp();
    // Test that Copy() method uses the optimized copy constructor
    let aCopiedGeom: Handle<dyn Geom2dGeometry> = myOriginalCurve.copy().unwrap();
    let aCopiedCurve: Handle<Geom2dBSplineCurve> = aCopiedGeom.cast::<Geom2dBSplineCurve>();

    assert!(!aCopiedCurve.is_null());

    // Verify the copy is functionally identical
    assert_eq!(myOriginalCurve.degree(), aCopiedCurve.degree());
    assert_eq!(myOriginalCurve.nb_poles(), aCopiedCurve.nb_poles());

    // Test evaluation at several points
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        let anOrigPnt: Point2 = myOriginalCurve.value(u).unwrap();
        let aCopyPnt: Point2 = aCopiedCurve.value(u).unwrap();
        assert!(anOrigPnt.is_equal(aCopyPnt, 1e-10));
        u += 0.25;
    }
}

#[test]
fn Geom2d_BSplineCurve_Test__RationalCurveCopyConstructor() {
    let _myOriginalCurve = SetUp();
    // Create a rational BSpline curve
    let aPoles = poles3(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );
    let aWeights = weights3(1.0, 2.0, 1.0);
    let (aKnots, aMults) = end_knots(3);

    let aRationalCurve = Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 2, false).unwrap(),
    );
    let aCopiedRational = Handle::new(Geom2dBSplineCurve::copy_of(&aRationalCurve).unwrap());

    assert!(aCopiedRational.is_rational());

    // Verify weights are copied correctly
    for i in 1..=aRationalCurve.nb_poles() {
        EXPECT_DOUBLE_EQ(
            aRationalCurve.weight(i).unwrap(),
            aCopiedRational.weight(i).unwrap(),
        );
    }
}

#[test]
fn Geom2d_BSplineCurve_Test__Evaluation_D0() {
    let myOriginalCurve = SetUp();
    let mut aPnt: Point2 = myOriginalCurve.d0(0.0).unwrap();
    assert!(aPnt.is_equal(Point2::new(0.0, 0.0), 1e-10));
    aPnt = myOriginalCurve.d0(1.0).unwrap();
    assert!(aPnt.is_equal(Point2::new(3.0, 0.0), 1e-10));
}

#[test]
fn Geom2d_BSplineCurve_Test__Evaluation_D1() {
    let myOriginalCurve = SetUp();
    let (_aPnt, aV1) = myOriginalCurve.d1(0.5).unwrap();
    assert!(aV1.magnitude() > 0.0);
}

#[test]
fn Geom2d_BSplineCurve_Test__Evaluation_D2() {
    let myOriginalCurve = SetUp();
    let (_aPnt, aV1, _aV2) = myOriginalCurve.d2(0.5).unwrap();
    assert!(aV1.magnitude() > 0.0);
}

#[test]
fn Geom2d_BSplineCurve_Test__Evaluation_DN() {
    let myOriginalCurve = SetUp();
    let aDN1: Vector2 = myOriginalCurve.dn(0.5, 1).unwrap();
    assert!(aDN1.magnitude() > 0.0);
    // DN(4) for degree 3 should be zero
    let aDN4: Vector2 = myOriginalCurve.dn(0.5, 4).unwrap();
    EXPECT_NEAR(aDN4.magnitude(), 0.0, 1e-10);
}

#[test]
fn Geom2d_BSplineCurve_Test__StartEndPoints() {
    let myOriginalCurve = SetUp();
    let aStart: Point2 = myOriginalCurve.start_point().unwrap();
    let anEnd: Point2 = myOriginalCurve.end_point().unwrap();
    assert!(aStart.is_equal(Point2::new(0.0, 0.0), 1e-10));
    assert!(anEnd.is_equal(Point2::new(3.0, 0.0), 1e-10));
}

#[test]
fn Geom2d_BSplineCurve_Test__Properties() {
    let myOriginalCurve = SetUp();
    assert_eq!(myOriginalCurve.degree(), 3);
    assert!(!myOriginalCurve.is_periodic());
    assert!(!myOriginalCurve.is_rational());
    assert!(!myOriginalCurve.is_closed().unwrap());
    assert!(myOriginalCurve.is_cn(3).unwrap());
}

#[test]
fn Geom2d_BSplineCurve_Test__SetPole() {
    let myOriginalCurve = SetUp();
    let aNewPole = Point2::new(1.0, 5.0);
    myOriginalCurve.set_pole(2, aNewPole).unwrap();
    assert!(myOriginalCurve.pole(2).unwrap().is_equal(aNewPole, 1e-10));
}

#[test]
fn Geom2d_BSplineCurve_Test__IncreaseDegree() {
    let myOriginalCurve = SetUp();
    let aValBefore: Point2 = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve.increase_degree(5).unwrap();
    assert_eq!(myOriginalCurve.degree(), 5);
    let aValAfter: Point2 = myOriginalCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(aValAfter, 1e-10));
}

#[test]
fn Geom2d_BSplineCurve_Test__InsertKnot() {
    let myOriginalCurve = SetUp();
    let aValBefore: Point2 = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve.insert_knot(0.5, 1, 0.0).unwrap();
    assert_eq!(myOriginalCurve.nb_knots(), 3);
    let aValAfter: Point2 = myOriginalCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(aValAfter, 1e-10));
}

#[test]
fn Geom2d_BSplineCurve_Test__RemoveKnot() {
    let myOriginalCurve = SetUp();
    myOriginalCurve.insert_knot(0.5, 1, 0.0).unwrap();
    assert_eq!(myOriginalCurve.nb_knots(), 3);

    let isRemoved: bool = myOriginalCurve.remove_knot(2, 0, 1e-6).unwrap();
    assert!(isRemoved);
    assert_eq!(myOriginalCurve.nb_knots(), 2);
}

#[test]
fn Geom2d_BSplineCurve_Test__Segment() {
    let myOriginalCurve = SetUp();
    let aP25: Point2 = myOriginalCurve.value(0.25).unwrap();
    let aP75: Point2 = myOriginalCurve.value(0.75).unwrap();

    myOriginalCurve
        .segment(0.25, 0.75, precision::p_confusion())
        .unwrap();
    EXPECT_NEAR(myOriginalCurve.first_parameter(), 0.25, 1e-10);
    EXPECT_NEAR(myOriginalCurve.last_parameter(), 0.75, 1e-10);
    assert!(myOriginalCurve.start_point().unwrap().is_equal(aP25, 1e-6));
    assert!(myOriginalCurve.end_point().unwrap().is_equal(aP75, 1e-6));
}

#[test]
fn Geom2d_BSplineCurve_Test__Reverse() {
    let myOriginalCurve = SetUp();
    let aStart: Point2 = myOriginalCurve.start_point().unwrap();
    let anEnd: Point2 = myOriginalCurve.end_point().unwrap();
    myOriginalCurve.reverse().unwrap();
    assert!(
        myOriginalCurve
            .start_point()
            .unwrap()
            .is_equal(anEnd, 1e-10)
    );
    assert!(myOriginalCurve.end_point().unwrap().is_equal(aStart, 1e-10));
}
