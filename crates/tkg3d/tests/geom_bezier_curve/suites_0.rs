// Geom_BezierCurve_Test.cxx, tests CopyConstructorBasicProperties to Transform.

#[test]
fn Geom_BezierCurve_Test__CopyConstructorBasicProperties() {
    let myOriginalCurve = SetUp();
    // Test copy constructor
    let aCopiedCurve = copied(&myOriginalCurve);

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
fn Geom_BezierCurve_Test__CopyConstructorPoles() {
    let myOriginalCurve = SetUp();
    let aCopiedCurve = copied(&myOriginalCurve);

    // Verify all poles are identical
    for i in 1..=myOriginalCurve.nb_poles() {
        let anOrigPole = myOriginalCurve.pole(i).unwrap();
        let aCopyPole = aCopiedCurve.pole(i).unwrap();
        assert!(anOrigPole.is_equal(&aCopyPole, 1e-10));
    }
}

#[test]
fn Geom_BezierCurve_Test__CopyMethodUsesOptimizedConstructor() {
    let myOriginalCurve = SetUp();
    // Test that Copy() method uses the optimized copy constructor
    let aCopiedGeom: Handle<dyn GeomGeometry> = myOriginalCurve.copy().unwrap();
    let aCopiedCurve: Handle<GeomBezierCurve> = aCopiedGeom.cast();

    assert!(!aCopiedCurve.is_null());

    // Verify the copy is functionally identical
    assert_eq!(myOriginalCurve.degree(), aCopiedCurve.degree());
    assert_eq!(myOriginalCurve.nb_poles(), aCopiedCurve.nb_poles());

    // Test evaluation at several points
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        let anOrigPnt = myOriginalCurve.value(u).unwrap();
        let aCopyPnt = aCopiedCurve.value(u).unwrap();
        assert!(anOrigPnt.is_equal(&aCopyPnt, 1e-10));
        u += 0.25;
    }
}

#[test]
fn Geom_BezierCurve_Test__RationalCurveCopyConstructor() {
    let _myOriginalCurve = SetUp();
    // Create a rational Bezier curve
    let aPoles = poles3(pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0));
    let aWeights = weights3(1.0, 2.0, 1.0);

    let aRationalCurve = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aCopiedRational = copied(&aRationalCurve);

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
fn Geom_BezierCurve_Test__CopyIndependence() {
    let myOriginalCurve = SetUp();
    let aCopiedCurve = copied(&myOriginalCurve);

    // Modify the original curve
    let aNewPole = pnt(10.0, 10.0, 10.0);
    myOriginalCurve.set_pole(2, aNewPole).unwrap();

    // Verify the copied curve is not affected
    let anOrigPole = aCopiedCurve.pole(2).unwrap();
    assert!(!anOrigPole.is_equal(&aNewPole, 1e-10));
}

#[test]
fn Geom_BezierCurve_Test__Evaluation_D0() {
    let myOriginalCurve = SetUp();
    let aPnt = myOriginalCurve.d0(0.0).unwrap();
    assert!(aPnt.is_equal(&pnt(0.0, 0.0, 0.0), 1e-10));
    let aPnt = myOriginalCurve.d0(1.0).unwrap();
    assert!(aPnt.is_equal(&pnt(3.0, 0.0, 0.0), 1e-10));
}

#[test]
fn Geom_BezierCurve_Test__Evaluation_D1() {
    let myOriginalCurve = SetUp();
    let (_aPnt, aV1) = myOriginalCurve.d1(0.5).unwrap();
    assert!(aV1.magnitude() > 0.0);
}

#[test]
fn Geom_BezierCurve_Test__Evaluation_D2() {
    let myOriginalCurve = SetUp();
    let (_aPnt, aV1, _aV2) = myOriginalCurve.d2(0.5).unwrap();
    assert!(aV1.magnitude() > 0.0);
}

#[test]
fn Geom_BezierCurve_Test__Evaluation_D3() {
    let myOriginalCurve = SetUp();
    let (_aPnt, _aV1, _aV2, aV3) = myOriginalCurve.d3(0.5).unwrap();
    // D3 of a degree 3 curve is constant
    let (_aPntB, _aV1b, _aV2b, aV3b) = myOriginalCurve.d3(0.25).unwrap();
    EXPECT_NEAR(aV3.x(), aV3b.x(), 1e-8);
    EXPECT_NEAR(aV3.y(), aV3b.y(), 1e-8);
}

#[test]
fn Geom_BezierCurve_Test__Evaluation_DN() {
    let myOriginalCurve = SetUp();
    let aDN1: Vector3 = myOriginalCurve.dn(0.5, 1).unwrap();
    assert!(aDN1.magnitude() > 0.0);
    // DN(4) for a degree 3 curve should be zero
    let aDN4: Vector3 = myOriginalCurve.dn(0.5, 4).unwrap();
    EXPECT_NEAR(aDN4.magnitude(), 0.0, 1e-10);
}

#[test]
fn Geom_BezierCurve_Test__StartEndPoints() {
    let myOriginalCurve = SetUp();
    let aStart = myOriginalCurve.start_point().unwrap();
    let anEnd = myOriginalCurve.end_point().unwrap();
    assert!(aStart.is_equal(&pnt(0.0, 0.0, 0.0), 1e-10));
    assert!(anEnd.is_equal(&pnt(3.0, 0.0, 0.0), 1e-10));
}

#[test]
fn Geom_BezierCurve_Test__Properties() {
    let myOriginalCurve = SetUp();
    assert_eq!(myOriginalCurve.degree(), 3);
    assert!(!myOriginalCurve.is_periodic());
    assert!(!myOriginalCurve.is_rational());
    assert!(!myOriginalCurve.is_closed().unwrap());
    assert!(myOriginalCurve.is_cn(100).unwrap());
    assert_eq!(myOriginalCurve.continuity(), Shape::CN);
    EXPECT_DOUBLE_EQ(myOriginalCurve.first_parameter(), 0.0);
    EXPECT_DOUBLE_EQ(myOriginalCurve.last_parameter(), 1.0);
}

#[test]
fn Geom_BezierCurve_Test__SetPole() {
    let myOriginalCurve = SetUp();
    let aNewPole = pnt(1.0, 5.0, 0.0);
    myOriginalCurve.set_pole(2, aNewPole).unwrap();
    assert!(myOriginalCurve.pole(2).unwrap().is_equal(&aNewPole, 1e-10));
    // Endpoints unchanged
    assert!(
        myOriginalCurve
            .start_point()
            .unwrap()
            .is_equal(&pnt(0.0, 0.0, 0.0), 1e-10)
    );
    assert!(
        myOriginalCurve
            .end_point()
            .unwrap()
            .is_equal(&pnt(3.0, 0.0, 0.0), 1e-10)
    );
}

#[test]
fn Geom_BezierCurve_Test__SetWeight() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0));
    let aWeights = weights3(1.0, 1.0, 1.0);

    let aCurve = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aMidBefore = aCurve.value(0.5).unwrap();

    aCurve.set_weight(2, 10.0).unwrap();
    EXPECT_DOUBLE_EQ(aCurve.weight(2).unwrap(), 10.0);
    assert!(aCurve.is_rational());

    let aMidAfter = aCurve.value(0.5).unwrap();
    // High weight pulls curve toward the control point
    assert!(aMidAfter.y() > aMidBefore.y());
}

#[test]
fn Geom_BezierCurve_Test__InsertPoleAfter() {
    let myOriginalCurve = SetUp();
    let aNbBefore = myOriginalCurve.nb_poles();
    let aNewPole = pnt(1.5, 2.0, 0.0);
    myOriginalCurve.insert_pole_after(2, aNewPole).unwrap();
    assert_eq!(myOriginalCurve.nb_poles(), aNbBefore + 1);
    assert_eq!(myOriginalCurve.degree(), 4);
    assert!(myOriginalCurve.pole(3).unwrap().is_equal(&aNewPole, 1e-10));
}

#[test]
fn Geom_BezierCurve_Test__InsertPoleBefore() {
    let myOriginalCurve = SetUp();
    let aNbBefore = myOriginalCurve.nb_poles();
    let aNewPole = pnt(0.5, 2.0, 0.0);
    myOriginalCurve.insert_pole_before(2, aNewPole).unwrap();
    assert_eq!(myOriginalCurve.nb_poles(), aNbBefore + 1);
    assert!(myOriginalCurve.pole(2).unwrap().is_equal(&aNewPole, 1e-10));
}

#[test]
fn Geom_BezierCurve_Test__RemovePole() {
    let myOriginalCurve = SetUp();
    // Add a pole first so we have 5, then remove one
    myOriginalCurve
        .insert_pole_after(2, pnt(1.5, 2.0, 0.0))
        .unwrap();
    assert_eq!(myOriginalCurve.nb_poles(), 5);

    myOriginalCurve.remove_pole(3).unwrap();
    assert_eq!(myOriginalCurve.nb_poles(), 4);
    assert_eq!(myOriginalCurve.degree(), 3);
}

#[test]
fn Geom_BezierCurve_Test__Increase() {
    let myOriginalCurve = SetUp();
    let aValBefore = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve.increase(5).unwrap();
    assert_eq!(myOriginalCurve.degree(), 5);
    assert_eq!(myOriginalCurve.nb_poles(), 6);
    let aValAfter = myOriginalCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BezierCurve_Test__Segment() {
    let myOriginalCurve = SetUp();
    let aP25 = myOriginalCurve.value(0.25).unwrap();
    let aP75 = myOriginalCurve.value(0.75).unwrap();

    myOriginalCurve.segment(0.25, 0.75).unwrap();
    assert!(myOriginalCurve.start_point().unwrap().is_equal(&aP25, 1e-6));
    assert!(myOriginalCurve.end_point().unwrap().is_equal(&aP75, 1e-6));
}

#[test]
fn Geom_BezierCurve_Test__Reverse() {
    let myOriginalCurve = SetUp();
    let aStart = myOriginalCurve.start_point().unwrap();
    let anEnd = myOriginalCurve.end_point().unwrap();
    myOriginalCurve.reverse().unwrap();
    assert!(myOriginalCurve.start_point().unwrap().is_equal(&anEnd, 1e-10));
    assert!(myOriginalCurve.end_point().unwrap().is_equal(&aStart, 1e-10));
}

#[test]
fn Geom_BezierCurve_Test__Resolution() {
    let myOriginalCurve = SetUp();
    let aUTol: f64 = myOriginalCurve.resolution(1.0).unwrap();
    assert!(aUTol > 0.0);
}

#[test]
fn Geom_BezierCurve_Test__Transform() {
    let myOriginalCurve = SetUp();
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(10.0, 20.0, 30.0));
    let aPtBefore = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve.transform(aTrsf).unwrap();
    let aPtAfter = myOriginalCurve.value(0.5).unwrap();
    EXPECT_NEAR(aPtAfter.x(), aPtBefore.x() + 10.0, 1e-10);
    EXPECT_NEAR(aPtAfter.y(), aPtBefore.y() + 20.0, 1e-10);
    EXPECT_NEAR(aPtAfter.z(), aPtBefore.z() + 30.0, 1e-10);
}
