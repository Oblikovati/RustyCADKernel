#[test]
fn Geom2d_BezierCurve_Test__CopyConstructorBasicProperties() {
    let myOriginalCurve = SetUp();
    // Test copy constructor
    let aCopiedCurve = Handle::new(Geom2dBezierCurve::copy_of(&myOriginalCurve).unwrap());

    // Verify basic properties are identical
    assert_eq!(myOriginalCurve.degree(), aCopiedCurve.degree());
    assert_eq!(myOriginalCurve.nb_poles(), aCopiedCurve.nb_poles());
    assert_eq!(myOriginalCurve.is_rational(), aCopiedCurve.is_rational());
    assert_eq!(
        myOriginalCurve.is_closed().unwrap(),
        aCopiedCurve.is_closed().unwrap()
    );
}

#[test]
fn Geom2d_BezierCurve_Test__CopyConstructorPoles() {
    let myOriginalCurve = SetUp();
    let aCopiedCurve = Handle::new(Geom2dBezierCurve::copy_of(&myOriginalCurve).unwrap());

    // Verify all poles are identical
    for i in 1..=myOriginalCurve.nb_poles() {
        let anOrigPole: Point2 = myOriginalCurve.pole(i).unwrap();
        let aCopyPole: Point2 = aCopiedCurve.pole(i).unwrap();
        assert!(anOrigPole.is_equal(aCopyPole, 1e-10));
    }
}

#[test]
fn Geom2d_BezierCurve_Test__CopyMethodUsesOptimizedConstructor() {
    let myOriginalCurve = SetUp();
    // Test that Copy() method uses the optimized copy constructor
    let aCopiedGeom: Handle<dyn Geom2dGeometry> = myOriginalCurve.copy().unwrap();
    let aCopiedCurve: Handle<Geom2dBezierCurve> = aCopiedGeom.cast::<Geom2dBezierCurve>();

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
fn Geom2d_BezierCurve_Test__RationalCurveCopyConstructor() {
    let _myOriginalCurve = SetUp();
    // Create a rational Bezier curve
    let aPoles = poles3(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );
    let aWeights = weights3(1.0, 2.0, 1.0);

    let aRationalCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aCopiedRational = Handle::new(Geom2dBezierCurve::copy_of(&aRationalCurve).unwrap());

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
fn Geom2d_BezierCurve_Test__CopyIndependence() {
    let myOriginalCurve = SetUp();
    let aCopiedCurve = Handle::new(Geom2dBezierCurve::copy_of(&myOriginalCurve).unwrap());

    // Modify the original curve
    let aNewPole = Point2::new(10.0, 10.0);
    myOriginalCurve.set_pole(2, aNewPole).unwrap();

    // Verify the copied curve is not affected
    let anOrigPole: Point2 = aCopiedCurve.pole(2).unwrap();
    assert!(!anOrigPole.is_equal(aNewPole, 1e-10));
}

#[test]
fn Geom2d_BezierCurve_Test__Evaluation_D0() {
    let myOriginalCurve = SetUp();
    let mut aPnt: Point2 = myOriginalCurve.d0(0.0).unwrap();
    assert!(aPnt.is_equal(Point2::new(0.0, 0.0), 1e-10));
    aPnt = myOriginalCurve.d0(1.0).unwrap();
    assert!(aPnt.is_equal(Point2::new(3.0, 0.0), 1e-10));
}

#[test]
fn Geom2d_BezierCurve_Test__Evaluation_D1() {
    let myOriginalCurve = SetUp();
    let (_aPnt, aV1) = myOriginalCurve.d1(0.5).unwrap();
    assert!(aV1.magnitude() > 0.0);
}

#[test]
fn Geom2d_BezierCurve_Test__Evaluation_D2() {
    let myOriginalCurve = SetUp();
    let (_aPnt, aV1, _aV2) = myOriginalCurve.d2(0.5).unwrap();
    assert!(aV1.magnitude() > 0.0);
}

#[test]
fn Geom2d_BezierCurve_Test__Evaluation_DN() {
    let myOriginalCurve = SetUp();
    let aDN1: Vector2 = myOriginalCurve.dn(0.5, 1).unwrap();
    assert!(aDN1.magnitude() > 0.0);
    let aDN4: Vector2 = myOriginalCurve.dn(0.5, 4).unwrap();
    EXPECT_NEAR(aDN4.magnitude(), 0.0, 1e-10);
}

#[test]
fn Geom2d_BezierCurve_Test__StartEndPoints() {
    let myOriginalCurve = SetUp();
    let aStart: Point2 = myOriginalCurve.start_point().unwrap();
    let anEnd: Point2 = myOriginalCurve.end_point().unwrap();
    assert!(aStart.is_equal(Point2::new(0.0, 0.0), 1e-10));
    assert!(anEnd.is_equal(Point2::new(3.0, 0.0), 1e-10));
}

#[test]
fn Geom2d_BezierCurve_Test__Properties() {
    let myOriginalCurve = SetUp();
    assert_eq!(myOriginalCurve.degree(), 3);
    assert!(!myOriginalCurve.is_periodic());
    assert!(!myOriginalCurve.is_rational());
    assert!(!myOriginalCurve.is_closed().unwrap());
    assert!(myOriginalCurve.is_cn(100).unwrap());
    EXPECT_DOUBLE_EQ(myOriginalCurve.first_parameter(), 0.0);
    EXPECT_DOUBLE_EQ(myOriginalCurve.last_parameter(), 1.0);
}

#[test]
fn Geom2d_BezierCurve_Test__SetPole() {
    let myOriginalCurve = SetUp();
    let aNewPole = Point2::new(1.0, 5.0);
    myOriginalCurve.set_pole(2, aNewPole).unwrap();
    assert!(myOriginalCurve.pole(2).unwrap().is_equal(aNewPole, 1e-10));
    assert!(
        myOriginalCurve
            .start_point()
            .unwrap()
            .is_equal(Point2::new(0.0, 0.0), 1e-10)
    );
}

#[test]
fn Geom2d_BezierCurve_Test__SetWeight() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );
    let aWeights = weights3(1.0, 1.0, 1.0);

    let aCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aMidBefore: Point2 = aCurve.value(0.5).unwrap();

    aCurve.set_weight(2, 10.0).unwrap();
    EXPECT_DOUBLE_EQ(aCurve.weight(2).unwrap(), 10.0);
    assert!(aCurve.is_rational());

    let aMidAfter: Point2 = aCurve.value(0.5).unwrap();
    assert!(aMidAfter.y() > aMidBefore.y());
}

#[test]
fn Geom2d_BezierCurve_Test__InsertPoleAfter() {
    let myOriginalCurve = SetUp();
    let aNbBefore: i32 = myOriginalCurve.nb_poles();
    myOriginalCurve
        .insert_pole_after(2, Point2::new(1.5, 2.0), 1.0)
        .unwrap();
    assert_eq!(myOriginalCurve.nb_poles(), aNbBefore + 1);
    assert_eq!(myOriginalCurve.degree(), 4);
}

#[test]
fn Geom2d_BezierCurve_Test__RemovePole() {
    let myOriginalCurve = SetUp();
    myOriginalCurve
        .insert_pole_after(2, Point2::new(1.5, 2.0), 1.0)
        .unwrap();
    assert_eq!(myOriginalCurve.nb_poles(), 5);

    myOriginalCurve.remove_pole(3).unwrap();
    assert_eq!(myOriginalCurve.nb_poles(), 4);
}

#[test]
fn Geom2d_BezierCurve_Test__Increase() {
    let myOriginalCurve = SetUp();
    let aValBefore: Point2 = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve.increase(5).unwrap();
    assert_eq!(myOriginalCurve.degree(), 5);
    let aValAfter: Point2 = myOriginalCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(aValAfter, 1e-10));
}

#[test]
fn Geom2d_BezierCurve_Test__Segment() {
    let myOriginalCurve = SetUp();
    let aP25: Point2 = myOriginalCurve.value(0.25).unwrap();
    let aP75: Point2 = myOriginalCurve.value(0.75).unwrap();

    myOriginalCurve.segment(0.25, 0.75).unwrap();
    assert!(myOriginalCurve.start_point().unwrap().is_equal(aP25, 1e-6));
    assert!(myOriginalCurve.end_point().unwrap().is_equal(aP75, 1e-6));
}

#[test]
fn Geom2d_BezierCurve_Test__Reverse() {
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
