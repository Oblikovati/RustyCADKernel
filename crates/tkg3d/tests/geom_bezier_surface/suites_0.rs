// Geom_BezierSurface_Test.cxx, tests CopyConstructorBasicProperties to Segment.

#[test]
fn Geom_BezierSurface_Test__CopyConstructorBasicProperties() {
    let myOriginalSurface = SetUp();
    // Test copy constructor
    let aCopiedSurface = copied(&myOriginalSurface);

    // Verify basic properties are identical
    assert_eq!(myOriginalSurface.u_degree(), aCopiedSurface.u_degree());
    assert_eq!(myOriginalSurface.v_degree(), aCopiedSurface.v_degree());
    assert_eq!(myOriginalSurface.nb_u_poles(), aCopiedSurface.nb_u_poles());
    assert_eq!(myOriginalSurface.nb_v_poles(), aCopiedSurface.nb_v_poles());
    assert_eq!(myOriginalSurface.is_u_rational(), aCopiedSurface.is_u_rational());
    assert_eq!(myOriginalSurface.is_v_rational(), aCopiedSurface.is_v_rational());
}

#[test]
fn Geom_BezierSurface_Test__CopyConstructorPoles() {
    let myOriginalSurface = SetUp();
    let aCopiedSurface = copied(&myOriginalSurface);

    // Verify all poles are identical
    for i in 1..=myOriginalSurface.nb_u_poles() {
        for j in 1..=myOriginalSurface.nb_v_poles() {
            let anOrigPole = myOriginalSurface.pole(i, j).unwrap();
            let aCopyPole = aCopiedSurface.pole(i, j).unwrap();
            assert!(anOrigPole.is_equal(&aCopyPole, 1e-10));
        }
    }
}

#[test]
fn Geom_BezierSurface_Test__CopyMethodUsesOptimizedConstructor() {
    let myOriginalSurface = SetUp();
    // Test that Copy() method uses the optimized copy constructor
    let aCopiedGeom: Handle<dyn GeomGeometry> = myOriginalSurface.copy().unwrap();
    let aCopiedSurface: Handle<GeomBezierSurface> = aCopiedGeom.cast();

    assert!(!aCopiedSurface.is_null());

    // Verify the copy is functionally identical
    assert_eq!(myOriginalSurface.u_degree(), aCopiedSurface.u_degree());
    assert_eq!(myOriginalSurface.v_degree(), aCopiedSurface.v_degree());

    // Test evaluation at several points
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        let mut v: f64 = 0.0;
        while v <= 1.0 {
            let anOrigPnt = myOriginalSurface.value(u, v).unwrap();
            let aCopyPnt = aCopiedSurface.value(u, v).unwrap();
            assert!(anOrigPnt.is_equal(&aCopyPnt, 1e-10));
            v += 0.5;
        }
        u += 0.5;
    }
}

#[test]
fn Geom_BezierSurface_Test__RationalSurfaceCopyConstructor() {
    let _myOriginalSurface = SetUp();
    // Create a rational Bezier surface
    let aPoles = unit_poles(0.0);
    let aWeights = weights22(1.0, 2.0, 2.0, 1.0);

    let aRationalSurface = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());
    let aCopiedRational = copied(&aRationalSurface);

    assert!(aCopiedRational.is_u_rational() || aCopiedRational.is_v_rational());

    // Verify weights are copied correctly
    for i in 1..=aRationalSurface.nb_u_poles() {
        for j in 1..=aRationalSurface.nb_v_poles() {
            EXPECT_DOUBLE_EQ(aRationalSurface.weight(i, j).unwrap(), aCopiedRational.weight(i, j).unwrap());
        }
    }
}

#[test]
fn Geom_BezierSurface_Test__CopyIndependence() {
    let myOriginalSurface = SetUp();
    let aCopiedSurface = copied(&myOriginalSurface);

    // Modify the original surface
    let aNewPole = pnt(10.0, 10.0, 10.0);
    myOriginalSurface.set_pole(2, 2, aNewPole).unwrap();

    // Verify the copied surface is not affected
    let anOrigPole = aCopiedSurface.pole(2, 2).unwrap();
    assert!(!anOrigPole.is_equal(&aNewPole, 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__Evaluation_D0() {
    let myOriginalSurface = SetUp();
    let aPnt = myOriginalSurface.d0(0.0, 0.0).unwrap();
    assert!(aPnt.is_equal(&pnt(1.0, 1.0, 0.2), 1e-10));
    let aPnt = myOriginalSurface.d0(1.0, 1.0).unwrap();
    assert!(aPnt.is_equal(&pnt(3.0, 3.0, 0.6), 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__Evaluation_D1() {
    let myOriginalSurface = SetUp();
    let aD1 = myOriginalSurface.d1(0.5, 0.5).unwrap();
    assert!(aD1.d1u.magnitude() > 0.0);
    assert!(aD1.d1v.magnitude() > 0.0);
}

#[test]
fn Geom_BezierSurface_Test__Evaluation_D2() {
    let myOriginalSurface = SetUp();
    let aD2 = myOriginalSurface.d2(0.5, 0.5).unwrap();
    assert!(aD2.d1u.magnitude() > 0.0);
}

#[test]
fn Geom_BezierSurface_Test__Evaluation_DN() {
    let myOriginalSurface = SetUp();
    let aDN10: Vector3 = myOriginalSurface.dn(0.5, 0.5, 1, 0).unwrap();
    assert!(aDN10.magnitude() > 0.0);
    let aDN01: Vector3 = myOriginalSurface.dn(0.5, 0.5, 0, 1).unwrap();
    assert!(aDN01.magnitude() > 0.0);
}

#[test]
fn Geom_BezierSurface_Test__Properties() {
    let myOriginalSurface = SetUp();
    assert_eq!(myOriginalSurface.u_degree(), 2);
    assert_eq!(myOriginalSurface.v_degree(), 2);
    assert!(!myOriginalSurface.is_u_periodic());
    assert!(!myOriginalSurface.is_v_periodic());
    assert!(!myOriginalSurface.is_u_rational());
    assert!(!myOriginalSurface.is_v_rational());
    assert_eq!(myOriginalSurface.nb_u_poles(), 3);
    assert_eq!(myOriginalSurface.nb_v_poles(), 3);
    assert!(myOriginalSurface.is_cn_u(100).unwrap());
    assert!(myOriginalSurface.is_cn_v(100).unwrap());
}

#[test]
fn Geom_BezierSurface_Test__Bounds() {
    let myOriginalSurface = SetUp();
    let (aU1, aU2, aV1, aV2) = myOriginalSurface.bounds();
    EXPECT_DOUBLE_EQ(aU1, 0.0);
    EXPECT_DOUBLE_EQ(aU2, 1.0);
    EXPECT_DOUBLE_EQ(aV1, 0.0);
    EXPECT_DOUBLE_EQ(aV2, 1.0);
}

#[test]
fn Geom_BezierSurface_Test__SetPole() {
    let myOriginalSurface = SetUp();
    let aNewPole = pnt(5.0, 5.0, 5.0);
    myOriginalSurface.set_pole(2, 2, aNewPole).unwrap();
    assert!(myOriginalSurface.pole(2, 2).unwrap().is_equal(&aNewPole, 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__SetWeight() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(0.0);
    let aWeights = weights22(1.0, 1.0, 1.0, 1.0);

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());
    let aValBefore = aSurf.value(0.5, 0.5).unwrap();

    aSurf.set_weight(2, 2, 10.0).unwrap();
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 2).unwrap(), 10.0);
    assert!(aSurf.is_u_rational() || aSurf.is_v_rational());

    let aValAfter = aSurf.value(0.5, 0.5).unwrap();
    assert!(!aValBefore.is_equal(&aValAfter, 1e-6));
}

#[test]
fn Geom_BezierSurface_Test__Increase() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.5, 0.5).unwrap();
    myOriginalSurface.increase(4, 3).unwrap();
    assert_eq!(myOriginalSurface.u_degree(), 4);
    assert_eq!(myOriginalSurface.v_degree(), 3);
    assert_eq!(myOriginalSurface.nb_u_poles(), 5);
    assert_eq!(myOriginalSurface.nb_v_poles(), 4);
    let aValAfter = myOriginalSurface.value(0.5, 0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__Segment() {
    let myOriginalSurface = SetUp();
    let aPntBefore = myOriginalSurface.value(0.5, 0.5).unwrap();
    myOriginalSurface.segment(0.25, 0.75, 0.25, 0.75).unwrap();

    assert!(myOriginalSurface.value(0.5, 0.5).unwrap().is_equal(&aPntBefore, 1e-6));
}
