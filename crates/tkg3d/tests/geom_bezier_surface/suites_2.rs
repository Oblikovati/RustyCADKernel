// Geom_BezierSurface_Test.cxx, tests InsertPoleRowBefore to WeightsArray_Rational_ReturnsOwning.

#[test]
fn Geom_BezierSurface_Test__InsertPoleRowBefore() {
    let myOriginalSurface = SetUp();
    let aNewRow = line(&[pnt(0.5, 1.0, 0.0), pnt(0.5, 2.0, 0.0), pnt(0.5, 3.0, 0.0)]);

    let aNbUBefore = myOriginalSurface.nb_u_poles();
    myOriginalSurface.insert_pole_row_before(2, &aNewRow).unwrap();
    assert_eq!(myOriginalSurface.nb_u_poles(), aNbUBefore + 1);
    assert!(myOriginalSurface.pole(2, 1).unwrap().is_equal(&pnt(0.5, 1.0, 0.0), 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__InsertPoleColBefore() {
    let myOriginalSurface = SetUp();
    let aNewCol = line(&[pnt(1.0, 0.5, 0.0), pnt(2.0, 0.5, 0.0), pnt(3.0, 0.5, 0.0)]);

    let aNbVBefore = myOriginalSurface.nb_v_poles();
    myOriginalSurface.insert_pole_col_before(2, &aNewCol).unwrap();
    assert_eq!(myOriginalSurface.nb_v_poles(), aNbVBefore + 1);
    assert!(myOriginalSurface.pole(1, 2).unwrap().is_equal(&pnt(1.0, 0.5, 0.0), 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__RationalSegment() {
    let _myOriginalSurface = SetUp();
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            let (x, y) = (f64::from(i), f64::from(j));
            aPoles[(i, j)] = pnt(x, y, (x + y) * 0.1);
            aWeights[(i, j)] = 1.0 + 0.3 * (f64::from(i - 1) + f64::from(j - 1));
        }
    }

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());
    assert!(aSurf.is_u_rational() || aSurf.is_v_rational());

    let aMid = aSurf.value(0.5, 0.5).unwrap();
    aSurf.segment(0.25, 0.75, 0.25, 0.75).unwrap();
    let aMidAfter = aSurf.value(0.5, 0.5).unwrap();
    assert!(aMid.is_equal(&aMidAfter, 1e-5));
}

#[test]
fn Geom_BezierSurface_Test__RationalIncrease() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(1.0);
    let aWeights = weights22(1.0, 2.0, 2.0, 1.0);

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());
    let aValBefore = aSurf.value(0.5, 0.5).unwrap();

    aSurf.increase(3, 3).unwrap();
    assert_eq!(aSurf.u_degree(), 3);
    assert_eq!(aSurf.v_degree(), 3);
    let aValAfter = aSurf.value(0.5, 0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BezierSurface_Test__SetWeightRow() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(0.0);
    let aWeights = weights22(1.0, 2.0, 3.0, 4.0);

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());

    let aNewWeights = line_weights(&[5.0, 6.0]);

    aSurf.set_weight_row(1, &aNewWeights).unwrap();
    EXPECT_DOUBLE_EQ(aSurf.weight(1, 1).unwrap(), 5.0);
    EXPECT_DOUBLE_EQ(aSurf.weight(1, 2).unwrap(), 6.0);
}

#[test]
fn Geom_BezierSurface_Test__SetWeightCol() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(0.0);
    let aWeights = weights22(1.0, 2.0, 3.0, 4.0);

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());

    let aNewWeights = line_weights(&[7.0, 8.0]);

    aSurf.set_weight_col(1, &aNewWeights).unwrap();
    EXPECT_DOUBLE_EQ(aSurf.weight(1, 1).unwrap(), 7.0);
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 1).unwrap(), 8.0);
}

#[test]
fn Geom_BezierSurface_Test__InsertPoleRowAfterWithWeights() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(0.0);
    let aWeights = weights22(1.0, 2.0, 2.0, 1.0);

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());

    let aNewRow = line(&[pnt(0.0, 0.5, 0.0), pnt(1.0, 0.5, 0.0)]);
    let aNewWeights = line_weights(&[3.0, 4.0]);

    aSurf.insert_pole_row_after_weighted(1, &aNewRow, &aNewWeights).unwrap();
    assert_eq!(aSurf.nb_u_poles(), 3);
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 1).unwrap(), 3.0);
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 2).unwrap(), 4.0);
}

#[test]
fn Geom_BezierSurface_Test__VIso_Rational() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(0.0);
    let aWeights = weights22(1.0, 2.0, 2.0, 1.0);

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());

    let anIso: Handle<dyn GeomCurve> = aSurf.v_iso(0.5).unwrap();
    assert!(!anIso.is_null());

    let mut u: f64 = 0.0;
    while u <= 1.0 {
        let aSurfPnt = aSurf.value(u, 0.5).unwrap();
        let anIsoPnt = anIso.value(u).unwrap();
        assert!(aSurfPnt.is_equal(&anIsoPnt, 1e-10));
        u += 0.25;
    }
}

#[test]
fn Geom_BezierSurface_Test__SetPoleWithWeight() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(0.0);
    let aWeights = weights22(1.0, 1.0, 1.0, 2.0);

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());

    let aNewPole = pnt(0.5, 0.5, 1.0);
    aSurf.set_pole_weighted(1, 1, aNewPole, 5.0).unwrap();
    assert!(aSurf.pole(1, 1).unwrap().is_equal(&aNewPole, 1e-10));
    EXPECT_DOUBLE_EQ(aSurf.weight(1, 1).unwrap(), 5.0);
}

#[test]
fn Geom_BezierSurface_Test__SetPoleColWithWeights() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(0.0);
    let aWeights = weights22(1.0, 2.0, 2.0, 1.0);

    let aSurf = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());

    let aNewCol = line(&[pnt(5.0, 0.0, 0.0), pnt(5.0, 1.0, 0.0)]);
    let aNewWeights = line_weights(&[3.0, 4.0]);

    aSurf.set_pole_col_weighted(2, &aNewCol, &aNewWeights).unwrap();
    assert!(aSurf.pole(1, 2).unwrap().is_equal(&pnt(5.0, 0.0, 0.0), 1e-10));
    EXPECT_DOUBLE_EQ(aSurf.weight(1, 2).unwrap(), 3.0);
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 2).unwrap(), 4.0);
}

#[test]
fn Geom_BezierSurface_Test__WeightsArray_NonRational_ReturnsUnitWeights() {
    let myOriginalSurface = SetUp();
    assert!(!myOriginalSurface.is_u_rational());
    assert!(!myOriginalSurface.is_v_rational());

    let aAddress = myOriginalSurface.weights_array(|aWeights| {
        assert_eq!(aWeights.nb_rows() as i32, myOriginalSurface.nb_u_poles());
        assert_eq!(aWeights.nb_columns() as i32, myOriginalSurface.nb_v_poles());
        assert!(!aWeights.is_deletable());
        for i in aWeights.lower_row()..=aWeights.upper_row() {
            for j in aWeights.lower_col()..=aWeights.upper_col() {
                EXPECT_DOUBLE_EQ(aWeights[(i, j)], 1.0);
            }
        }
        std::ptr::from_ref(aWeights)
    });
    assert_eq!(aAddress, myOriginalSurface.weights_array(std::ptr::from_ref));
}

#[test]
fn Geom_BezierSurface_Test__WeightsArray_Rational_ReturnsOwning() {
    let _myOriginalSurface = SetUp();
    let aPoles = unit_poles(0.0);
    let aWeightsIn = weights22(1.0, 2.0, 1.0, 1.0);

    let aRational = Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeightsIn).unwrap());

    let aAddress = aRational.weights_array(|aWeights| {
        assert_eq!(aWeights.len(), 4);
        assert!(aWeights.is_deletable());
        EXPECT_DOUBLE_EQ(aWeights[(1, 2)], 2.0);
        EXPECT_DOUBLE_EQ(aWeights[(1, 1)], 1.0);
        std::ptr::from_ref(aWeights)
    });
    assert_eq!(aAddress, aRational.weights_array(std::ptr::from_ref));
}
