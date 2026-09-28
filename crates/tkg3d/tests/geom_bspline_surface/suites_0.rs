// Geom_BSplineSurface_Test.cxx, tests CopyConstructorBasicProperties to KnotsAccess.

#[test]
fn Geom_BSplineSurface_Test__CopyConstructorBasicProperties() {
    let myOriginalSurface = SetUp();
    // Test copy constructor
    let aCopiedSurface = copied(&myOriginalSurface);

    // Verify basic properties are identical
    assert_eq!(myOriginalSurface.u_degree(), aCopiedSurface.u_degree());
    assert_eq!(myOriginalSurface.v_degree(), aCopiedSurface.v_degree());
    assert_eq!(myOriginalSurface.nb_u_poles(), aCopiedSurface.nb_u_poles());
    assert_eq!(myOriginalSurface.nb_v_poles(), aCopiedSurface.nb_v_poles());
    assert_eq!(myOriginalSurface.nb_u_knots(), aCopiedSurface.nb_u_knots());
    assert_eq!(myOriginalSurface.nb_v_knots(), aCopiedSurface.nb_v_knots());
    assert_eq!(myOriginalSurface.is_u_periodic(), aCopiedSurface.is_u_periodic());
    assert_eq!(myOriginalSurface.is_v_periodic(), aCopiedSurface.is_v_periodic());
}

#[test]
fn Geom_BSplineSurface_Test__CopyConstructorPoles() {
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
fn Geom_BSplineSurface_Test__CopyConstructorKnots() {
    let myOriginalSurface = SetUp();
    let aCopiedSurface = copied(&myOriginalSurface);

    // Verify U knots are identical
    for i in 1..=myOriginalSurface.nb_u_knots() {
        EXPECT_DOUBLE_EQ(
            myOriginalSurface.u_knot(i).unwrap(),
            aCopiedSurface.u_knot(i).unwrap(),
        );
        assert_eq!(
            myOriginalSurface.u_multiplicity(i).unwrap(),
            aCopiedSurface.u_multiplicity(i).unwrap()
        );
    }

    // Verify V knots are identical
    for i in 1..=myOriginalSurface.nb_v_knots() {
        EXPECT_DOUBLE_EQ(
            myOriginalSurface.v_knot(i).unwrap(),
            aCopiedSurface.v_knot(i).unwrap(),
        );
        assert_eq!(
            myOriginalSurface.v_multiplicity(i).unwrap(),
            aCopiedSurface.v_multiplicity(i).unwrap()
        );
    }
}

#[test]
fn Geom_BSplineSurface_Test__CopyMethodUsesOptimizedConstructor() {
    let myOriginalSurface = SetUp();
    // Test that Copy() method uses the optimized copy constructor
    let aCopiedGeom: Handle<dyn GeomGeometry> = myOriginalSurface.copy().unwrap();
    let aCopiedSurface: Handle<GeomBSplineSurface> = aCopiedGeom.cast();

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
fn Geom_BSplineSurface_Test__CopyIndependence() {
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
fn Geom_BSplineSurface_Test__Evaluation_D0() {
    let myOriginalSurface = SetUp();
    let mut aPnt = myOriginalSurface.d0(0.0, 0.0).unwrap();
    assert!(aPnt.is_equal(&pnt(1.0, 1.0, 0.2), 1e-10));
    aPnt = myOriginalSurface.d0(1.0, 1.0).unwrap();
    assert!(aPnt.is_equal(&pnt(3.0, 3.0, 0.6), 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__Evaluation_D1() {
    let myOriginalSurface = SetUp();
    let aResult = myOriginalSurface.d1(0.5, 0.5).unwrap();
    assert!(aResult.d1u.magnitude() > 0.0);
    assert!(aResult.d1v.magnitude() > 0.0);
}

#[test]
fn Geom_BSplineSurface_Test__Evaluation_D2() {
    let myOriginalSurface = SetUp();
    let aResult = myOriginalSurface.d2(0.5, 0.5).unwrap();
    assert!(aResult.d1u.magnitude() > 0.0);
}

#[test]
fn Geom_BSplineSurface_Test__Evaluation_DN() {
    let myOriginalSurface = SetUp();
    let aDN10: Vector3 = myOriginalSurface.dn(0.5, 0.5, 1, 0).unwrap();
    assert!(aDN10.magnitude() > 0.0);
    let aDN01: Vector3 = myOriginalSurface.dn(0.5, 0.5, 0, 1).unwrap();
    assert!(aDN01.magnitude() > 0.0);
}

#[test]
fn Geom_BSplineSurface_Test__Properties() {
    let myOriginalSurface = SetUp();
    assert_eq!(myOriginalSurface.u_degree(), 2);
    assert_eq!(myOriginalSurface.v_degree(), 2);
    assert!(!myOriginalSurface.is_u_periodic());
    assert!(!myOriginalSurface.is_v_periodic());
    assert!(!myOriginalSurface.is_u_rational());
    assert!(!myOriginalSurface.is_v_rational());
    assert_eq!(myOriginalSurface.nb_u_poles(), 3);
    assert_eq!(myOriginalSurface.nb_v_poles(), 3);
    assert_eq!(myOriginalSurface.nb_u_knots(), 2);
    assert_eq!(myOriginalSurface.nb_v_knots(), 2);
}

#[test]
fn Geom_BSplineSurface_Test__Bounds() {
    let myOriginalSurface = SetUp();
    let (aU1, aU2, aV1, aV2) = myOriginalSurface.bounds();
    EXPECT_DOUBLE_EQ(aU1, 0.0);
    EXPECT_DOUBLE_EQ(aU2, 1.0);
    EXPECT_DOUBLE_EQ(aV1, 0.0);
    EXPECT_DOUBLE_EQ(aV2, 1.0);
}

#[test]
fn Geom_BSplineSurface_Test__SetPole() {
    let myOriginalSurface = SetUp();
    let aNewPole = pnt(5.0, 5.0, 5.0);
    myOriginalSurface.set_pole(2, 2, aNewPole).unwrap();
    assert!(myOriginalSurface.pole(2, 2).unwrap().is_equal(&aNewPole, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__SetWeight() {
    let _myOriginalSurface = SetUp();
    // Create a rational surface with non-planar poles and non-uniform weights
    let (aPoles, aWeights) = raised_centre();
    let aSurf = rational_patch(&aPoles, &aWeights);
    assert!(aSurf.is_u_rational() || aSurf.is_v_rational());

    let aValBefore = aSurf.value(0.5, 0.5).unwrap();
    aSurf.set_weight(2, 2, 10.0).unwrap();
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 2).unwrap(), 10.0);
    let aValAfter = aSurf.value(0.5, 0.5).unwrap();
    // Higher weight at the elevated center pole pulls surface up
    assert!(aValAfter.z() > aValBefore.z());
}

#[test]
fn Geom_BSplineSurface_Test__IncreaseDegree() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.5, 0.5).unwrap();
    myOriginalSurface.increase_degree(4, 3).unwrap();
    assert_eq!(myOriginalSurface.u_degree(), 4);
    assert_eq!(myOriginalSurface.v_degree(), 3);
    let aValAfter = myOriginalSurface.value(0.5, 0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__InsertUKnot() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.5, 0.5).unwrap();
    myOriginalSurface.insert_u_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalSurface.nb_u_knots(), 3);
    let aValAfter = myOriginalSurface.value(0.5, 0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__InsertVKnot() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.5, 0.5).unwrap();
    myOriginalSurface.insert_v_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalSurface.nb_v_knots(), 3);
    let aValAfter = myOriginalSurface.value(0.5, 0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__RemoveUKnot() {
    let myOriginalSurface = SetUp();
    myOriginalSurface.insert_u_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalSurface.nb_u_knots(), 3);

    let aValBefore = myOriginalSurface.value(0.25, 0.5).unwrap();
    let isRemoved: bool = myOriginalSurface.remove_u_knot(2, 0, 1e-6).unwrap();
    assert!(isRemoved);
    assert_eq!(myOriginalSurface.nb_u_knots(), 2);
    let aValAfter = myOriginalSurface.value(0.25, 0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-6));
}

#[test]
fn Geom_BSplineSurface_Test__Segment() {
    let myOriginalSurface = SetUp();
    let aPntBefore = myOriginalSurface.value(0.5, 0.5).unwrap();
    // Default tolerances: Precision::PConfusion() in both directions.
    let aTol: f64 = rustycad_tkernel::precision::p_confusion();
    myOriginalSurface
        .segment((0.25, 0.75), (0.25, 0.75), (aTol, aTol))
        .unwrap();

    let (aU1, aU2, aV1, aV2) = myOriginalSurface.bounds();
    EXPECT_NEAR(aU1, 0.25, 1e-10);
    EXPECT_NEAR(aU2, 0.75, 1e-10);
    EXPECT_NEAR(aV1, 0.25, 1e-10);
    EXPECT_NEAR(aV2, 0.75, 1e-10);

    let aPntAfter = myOriginalSurface.value(0.5, 0.5).unwrap();
    assert!(aPntBefore.is_equal(&aPntAfter, 1e-6));
}

#[test]
fn Geom_BSplineSurface_Test__ExchangeUV() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.3, 0.7).unwrap();
    let aUDeg: i32 = myOriginalSurface.u_degree();
    let aVDeg: i32 = myOriginalSurface.v_degree();

    myOriginalSurface.exchange_uv().unwrap();
    assert_eq!(myOriginalSurface.u_degree(), aVDeg);
    assert_eq!(myOriginalSurface.v_degree(), aUDeg);

    let aValAfter = myOriginalSurface.value(0.7, 0.3).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__UReverse() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.3, 0.5).unwrap();
    myOriginalSurface.u_reverse().unwrap();
    let aValAfter = myOriginalSurface.value(0.7, 0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__VReverse() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.5, 0.3).unwrap();
    myOriginalSurface.v_reverse().unwrap();
    let aValAfter = myOriginalSurface.value(0.5, 0.7).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__UIso() {
    let myOriginalSurface = SetUp();
    let anIso: Handle<dyn GeomCurve> = myOriginalSurface.u_iso(0.5).unwrap();
    assert!(!anIso.is_null());

    // UIso at u=0.5 should match surface evaluation
    let mut v: f64 = 0.0;
    while v <= 1.0 {
        let aSurfPnt = myOriginalSurface.value(0.5, v).unwrap();
        let anIsoPnt = anIso.value(v).unwrap();
        assert!(aSurfPnt.is_equal(&anIsoPnt, 1e-10));
        v += 0.25;
    }
}

#[test]
fn Geom_BSplineSurface_Test__VIso() {
    let myOriginalSurface = SetUp();
    let anIso: Handle<dyn GeomCurve> = myOriginalSurface.v_iso(0.5).unwrap();
    assert!(!anIso.is_null());

    let mut u: f64 = 0.0;
    while u <= 1.0 {
        let aSurfPnt = myOriginalSurface.value(u, 0.5).unwrap();
        let anIsoPnt = anIso.value(u).unwrap();
        assert!(aSurfPnt.is_equal(&anIsoPnt, 1e-10));
        u += 0.25;
    }
}

#[test]
fn Geom_BSplineSurface_Test__Resolution() {
    let myOriginalSurface = SetUp();
    let (aUTol, aVTol) = myOriginalSurface.resolution(1.0).unwrap();
    assert!(aUTol > 0.0);
    assert!(aVTol > 0.0);
}

#[test]
fn Geom_BSplineSurface_Test__Transform() {
    let myOriginalSurface = SetUp();
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(10.0, 20.0, 30.0));
    let aPtBefore = myOriginalSurface.value(0.5, 0.5).unwrap();
    myOriginalSurface.transform(aTrsf).unwrap();
    let aPtAfter = myOriginalSurface.value(0.5, 0.5).unwrap();
    EXPECT_NEAR(aPtAfter.x(), aPtBefore.x() + 10.0, 1e-10);
    EXPECT_NEAR(aPtAfter.y(), aPtBefore.y() + 20.0, 1e-10);
    EXPECT_NEAR(aPtAfter.z(), aPtBefore.z() + 30.0, 1e-10);
}

#[test]
fn Geom_BSplineSurface_Test__KnotsAccess() {
    let myOriginalSurface = SetUp();
    let aUKnots = myOriginalSurface.u_knots();
    assert_eq!(aUKnots.len(), 2);
    EXPECT_DOUBLE_EQ(aUKnots[1], 0.0);
    EXPECT_DOUBLE_EQ(aUKnots[2], 1.0);

    let aVKnots = myOriginalSurface.v_knots();
    assert_eq!(aVKnots.len(), 2);

    let aUMults = myOriginalSurface.u_multiplicities();
    assert_eq!(aUMults.len(), 2);
    assert_eq!(aUMults[1], 3);

    let aVMults = myOriginalSurface.v_multiplicities();
    assert_eq!(aVMults.len(), 2);
}
