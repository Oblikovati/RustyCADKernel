// Geom_BezierSurface_Test.cxx, tests ExchangeUV to Evaluation_D3.

#[test]
fn Geom_BezierSurface_Test__ExchangeUV() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.3, 0.7).unwrap();
    let aUDeg = myOriginalSurface.u_degree();
    let aVDeg = myOriginalSurface.v_degree();

    myOriginalSurface.exchange_uv().unwrap();
    assert_eq!(myOriginalSurface.u_degree(), aVDeg);
    assert_eq!(myOriginalSurface.v_degree(), aUDeg);

    let aValAfter = myOriginalSurface.value(0.7, 0.3).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__UReverse() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.3, 0.5).unwrap();
    myOriginalSurface.u_reverse().unwrap();
    let aValAfter = myOriginalSurface.value(0.7, 0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__VReverse() {
    let myOriginalSurface = SetUp();
    let aValBefore = myOriginalSurface.value(0.5, 0.3).unwrap();
    myOriginalSurface.v_reverse().unwrap();
    let aValAfter = myOriginalSurface.value(0.5, 0.7).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__UIso() {
    let myOriginalSurface = SetUp();
    let anIso: Handle<dyn GeomCurve> = myOriginalSurface.u_iso(0.5).unwrap();
    assert!(!anIso.is_null());

    let mut v: f64 = 0.0;
    while v <= 1.0 {
        let aSurfPnt = myOriginalSurface.value(0.5, v).unwrap();
        let anIsoPnt = anIso.value(v).unwrap();
        assert!(aSurfPnt.is_equal(&anIsoPnt, 1e-10));
        v += 0.25;
    }
}

#[test]
fn Geom_BezierSurface_Test__VIso() {
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
fn Geom_BezierSurface_Test__Resolution() {
    let myOriginalSurface = SetUp();
    let (aUTol, aVTol) = myOriginalSurface.resolution(1.0).unwrap();
    assert!(aUTol > 0.0);
    assert!(aVTol > 0.0);
}

#[test]
fn Geom_BezierSurface_Test__Transform() {
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
fn Geom_BezierSurface_Test__InsertPoleRowAfter() {
    let myOriginalSurface = SetUp();
    let aNewRow = line(&[pnt(1.5, 1.0, 0.0), pnt(1.5, 2.0, 0.0), pnt(1.5, 3.0, 0.0)]);

    let aNbUBefore = myOriginalSurface.nb_u_poles();
    myOriginalSurface.insert_pole_row_after(1, &aNewRow).unwrap();
    assert_eq!(myOriginalSurface.nb_u_poles(), aNbUBefore + 1);
    assert_eq!(myOriginalSurface.u_degree(), 3);
}

#[test]
fn Geom_BezierSurface_Test__InsertPoleColAfter() {
    let myOriginalSurface = SetUp();
    let aNewCol = line(&[pnt(1.0, 1.5, 0.0), pnt(2.0, 1.5, 0.0), pnt(3.0, 1.5, 0.0)]);

    let aNbVBefore = myOriginalSurface.nb_v_poles();
    myOriginalSurface.insert_pole_col_after(1, &aNewCol).unwrap();
    assert_eq!(myOriginalSurface.nb_v_poles(), aNbVBefore + 1);
    assert_eq!(myOriginalSurface.v_degree(), 3);
}

#[test]
fn Geom_BezierSurface_Test__RemovePoleRow() {
    let myOriginalSurface = SetUp();
    // Insert a row first so we have 4 rows (need at least 2 to remove)
    let aNewRow = line(&[pnt(1.5, 1.0, 0.0), pnt(1.5, 2.0, 0.0), pnt(1.5, 3.0, 0.0)]);
    myOriginalSurface.insert_pole_row_after(1, &aNewRow).unwrap();
    assert_eq!(myOriginalSurface.nb_u_poles(), 4);

    myOriginalSurface.remove_pole_row(2).unwrap();
    assert_eq!(myOriginalSurface.nb_u_poles(), 3);
}

#[test]
fn Geom_BezierSurface_Test__RemovePoleCol() {
    let myOriginalSurface = SetUp();
    let aNewCol = line(&[pnt(1.0, 1.5, 0.0), pnt(2.0, 1.5, 0.0), pnt(3.0, 1.5, 0.0)]);
    myOriginalSurface.insert_pole_col_after(1, &aNewCol).unwrap();
    assert_eq!(myOriginalSurface.nb_v_poles(), 4);

    myOriginalSurface.remove_pole_col(2).unwrap();
    assert_eq!(myOriginalSurface.nb_v_poles(), 3);
}

#[test]
fn Geom_BezierSurface_Test__SetPoleRow() {
    let myOriginalSurface = SetUp();
    let aNewRow = line(&[pnt(10.0, 1.0, 0.0), pnt(10.0, 2.0, 0.0), pnt(10.0, 3.0, 0.0)]);

    myOriginalSurface.set_pole_row(2, &aNewRow).unwrap();
    for j in 1..=3 {
        EXPECT_NEAR(myOriginalSurface.pole(2, j).unwrap().x(), 10.0, 1e-10);
    }
}

#[test]
fn Geom_BezierSurface_Test__SetPoleCol() {
    let myOriginalSurface = SetUp();
    let aNewCol = line(&[pnt(1.0, 10.0, 0.0), pnt(2.0, 10.0, 0.0), pnt(3.0, 10.0, 0.0)]);

    myOriginalSurface.set_pole_col(2, &aNewCol).unwrap();
    for i in 1..=3 {
        EXPECT_NEAR(myOriginalSurface.pole(i, 2).unwrap().y(), 10.0, 1e-10);
    }
}

#[test]
fn Geom_BezierSurface_Test__WeightsAccess_NonRational() {
    let myOriginalSurface = SetUp();
    let aWeights = myOriginalSurface.weights();
    assert!(aWeights.is_none());
}

#[test]
fn Geom_BezierSurface_Test__PolesAccess() {
    let myOriginalSurface = SetUp();
    let aPoles = myOriginalSurface.poles();
    assert_eq!(aPoles.nb_rows(), 3);
    assert_eq!(aPoles.nb_columns(), 3);
    assert!(aPoles[(1, 1)].is_equal(&pnt(1.0, 1.0, 0.2), 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__MaxDegree() {
    let _myOriginalSurface = SetUp();
    assert!(GeomBezierSurface::max_degree() >= 25);
}

#[test]
fn Geom_BezierSurface_Test__RationalSurface_UIso() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(0.0);
    let aWeights = weights22(1.0, 2.0, 2.0, 1.0);

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());

    let anIso: Handle<dyn GeomCurve> = aSurf.u_iso(0.5).unwrap();
    assert!(!anIso.is_null());

    let mut v: f64 = 0.0;
    while v <= 1.0 {
        let aSurfPnt = aSurf.value(0.5, v).unwrap();
        let anIsoPnt = anIso.value(v).unwrap();
        assert!(aSurfPnt.is_equal(&anIsoPnt, 1e-10));
        v += 0.25;
    }
}

#[test]
fn Geom_BezierSurface_Test__Evaluation_D3() {
    let myOriginalSurface = SetUp();
    let aD3 = myOriginalSurface.d3(0.5, 0.5).unwrap();
    // Degree 2: D3 should be zero
    EXPECT_NEAR(aD3.d3u.magnitude(), 0.0, 1e-10);
    EXPECT_NEAR(aD3.d3v.magnitude(), 0.0, 1e-10);
}
