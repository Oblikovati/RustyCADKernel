// Geom_BSplineCurve_Test.cxx, tests CopyConstructorBasicProperties to Transform.

#[test]
fn Geom_BSplineCurve_Test__CopyConstructorBasicProperties() {
    let myOriginalCurve = SetUp();
    // Test copy constructor
    let aCopiedCurve = copied(&myOriginalCurve);

    // Verify basic properties are identical
    assert_eq!(myOriginalCurve.degree(), aCopiedCurve.degree());
    assert_eq!(myOriginalCurve.nb_poles(), aCopiedCurve.nb_poles());
    assert_eq!(myOriginalCurve.nb_knots(), aCopiedCurve.nb_knots());
    assert_eq!(myOriginalCurve.is_periodic(), aCopiedCurve.is_periodic());
    assert_eq!(myOriginalCurve.is_rational(), aCopiedCurve.is_rational());
}

#[test]
fn Geom_BSplineCurve_Test__CopyConstructorPoles() {
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
fn Geom_BSplineCurve_Test__CopyConstructorKnots() {
    let myOriginalCurve = SetUp();
    let aCopiedCurve = copied(&myOriginalCurve);

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
fn Geom_BSplineCurve_Test__CopyMethodUsesOptimizedConstructor() {
    let myOriginalCurve = SetUp();
    // Test that Copy() method uses the optimized copy constructor
    let aCopiedGeom: Handle<dyn GeomGeometry> = myOriginalCurve.copy().unwrap();
    let aCopiedCurve: Handle<GeomBSplineCurve> = aCopiedGeom.cast();

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
fn Geom_BSplineCurve_Test__RationalCurveCopyConstructor() {
    let _myOriginalCurve = SetUp();
    // Create a rational BSpline curve
    let aRationalCurve = rational_quadratic(
        [pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0)],
        2.0,
    );
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
fn Geom_BSplineCurve_Test__Evaluation_D0() {
    let myOriginalCurve = SetUp();
    let aPnt = myOriginalCurve.d0(0.0).unwrap();
    assert!(aPnt.is_equal(&pnt(0.0, 0.0, 0.0), 1e-10));
    let aPnt = myOriginalCurve.d0(1.0).unwrap();
    assert!(aPnt.is_equal(&pnt(3.0, 0.0, 0.0), 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__Evaluation_D1() {
    let myOriginalCurve = SetUp();
    let (_aPnt, aV1) = myOriginalCurve.d1(0.5).unwrap();
    assert!(aV1.magnitude() > 0.0);
}

#[test]
fn Geom_BSplineCurve_Test__Evaluation_D2() {
    let myOriginalCurve = SetUp();
    let (_aPnt, aV1, _aV2) = myOriginalCurve.d2(0.5).unwrap();
    assert!(aV1.magnitude() > 0.0);
}

#[test]
fn Geom_BSplineCurve_Test__Evaluation_D3() {
    let myOriginalCurve = SetUp();
    let (_aPnt, _aV1, _aV2, aV3) = myOriginalCurve.d3(0.5).unwrap();
    // D3 of a degree 3 curve is constant
    let (_aPntB, _aV1b, _aV2b, aV3b) = myOriginalCurve.d3(0.25).unwrap();
    EXPECT_NEAR(aV3.x(), aV3b.x(), 1e-8);
    EXPECT_NEAR(aV3.y(), aV3b.y(), 1e-8);
    EXPECT_NEAR(aV3.z(), aV3b.z(), 1e-8);
}

#[test]
fn Geom_BSplineCurve_Test__Evaluation_DN() {
    let myOriginalCurve = SetUp();
    let aDN: Vector3 = myOriginalCurve.dn(0.5, 1).unwrap();
    assert!(aDN.magnitude() > 0.0);
    // DN(4) for a degree 3 curve should be zero
    let aDN4: Vector3 = myOriginalCurve.dn(0.5, 4).unwrap();
    EXPECT_NEAR(aDN4.magnitude(), 0.0, 1e-10);
}

#[test]
fn Geom_BSplineCurve_Test__StartEndPoints() {
    let myOriginalCurve = SetUp();
    let aStart = myOriginalCurve.start_point().unwrap();
    let anEnd = myOriginalCurve.end_point().unwrap();
    assert!(aStart.is_equal(&pnt(0.0, 0.0, 0.0), 1e-10));
    assert!(anEnd.is_equal(&pnt(3.0, 0.0, 0.0), 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__SetPole() {
    let myOriginalCurve = SetUp();
    let aNewPole = pnt(1.0, 2.0, 3.0);
    myOriginalCurve.set_pole(2, aNewPole).unwrap();
    assert!(myOriginalCurve.pole(2).unwrap().is_equal(&aNewPole, 1e-10));
    // End points should not change
    assert!(
        myOriginalCurve
            .start_point()
            .unwrap()
            .is_equal(&pnt(0.0, 0.0, 0.0), 1e-10)
    );
}

#[test]
fn Geom_BSplineCurve_Test__SetWeight() {
    let _myOriginalCurve = SetUp();
    // Create rational curve
    let aCurve = rational_quadratic(
        [pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0)],
        1.0,
    );
    let aMidBefore = aCurve.value(0.5).unwrap();

    aCurve.set_weight(2, 5.0).unwrap();
    EXPECT_DOUBLE_EQ(aCurve.weight(2).unwrap(), 5.0);
    assert!(aCurve.is_rational());

    let aMidAfter = aCurve.value(0.5).unwrap();
    assert!(!aMidBefore.is_equal(&aMidAfter, 1e-6));
}

#[test]
fn Geom_BSplineCurve_Test__IncreaseDegree() {
    let myOriginalCurve = SetUp();
    let aValBefore = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve.increase_degree(5).unwrap();
    assert_eq!(myOriginalCurve.degree(), 5);
    let aValAfter = myOriginalCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__InsertKnot() {
    let myOriginalCurve = SetUp();
    let aValBefore = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve.insert_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalCurve.nb_knots(), 3);
    let aValAfter = myOriginalCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__RemoveKnot() {
    let myOriginalCurve = SetUp();
    myOriginalCurve.insert_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalCurve.nb_knots(), 3);
    let aValBefore = myOriginalCurve.value(0.25).unwrap();

    let isRemoved: bool = myOriginalCurve.remove_knot(2, 0, 1e-6).unwrap();
    assert!(isRemoved);
    assert_eq!(myOriginalCurve.nb_knots(), 2);
    let aValAfter = myOriginalCurve.value(0.25).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-6));
}

#[test]
fn Geom_BSplineCurve_Test__Segment() {
    let myOriginalCurve = SetUp();
    let aP25 = myOriginalCurve.value(0.25).unwrap();
    let aP75 = myOriginalCurve.value(0.75).unwrap();

    myOriginalCurve
        .segment(0.25, 0.75, precision::p_confusion())
        .unwrap();
    EXPECT_NEAR(myOriginalCurve.first_parameter(), 0.25, 1e-10);
    EXPECT_NEAR(myOriginalCurve.last_parameter(), 0.75, 1e-10);
    assert!(myOriginalCurve.start_point().unwrap().is_equal(&aP25, 1e-6));
    assert!(myOriginalCurve.end_point().unwrap().is_equal(&aP75, 1e-6));
}

#[test]
fn Geom_BSplineCurve_Test__Reverse() {
    let myOriginalCurve = SetUp();
    let aStart = myOriginalCurve.start_point().unwrap();
    let anEnd = myOriginalCurve.end_point().unwrap();
    myOriginalCurve.reverse().unwrap();
    assert!(myOriginalCurve.start_point().unwrap().is_equal(&anEnd, 1e-10));
    assert!(myOriginalCurve.end_point().unwrap().is_equal(&aStart, 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__Resolution() {
    let myOriginalCurve = SetUp();
    let aUTol: f64 = myOriginalCurve.resolution(1.0).unwrap();
    assert!(aUTol > 0.0);
}

#[test]
fn Geom_BSplineCurve_Test__Properties() {
    let myOriginalCurve = SetUp();
    assert_eq!(myOriginalCurve.degree(), 3);
    assert!(!myOriginalCurve.is_periodic());
    assert!(!myOriginalCurve.is_rational());
    assert!(!myOriginalCurve.is_closed().unwrap());
    assert!(myOriginalCurve.is_cn(3).unwrap());
    assert_eq!(myOriginalCurve.continuity(), Shape::CN);
}

#[test]
fn Geom_BSplineCurve_Test__Transform() {
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
