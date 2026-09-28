// Geom_BSplineSurface_Test.cxx, tests WeightsAccess_NonRational to OCC30990_CacheConsistencyAtKnots.

#[test]
fn Geom_BSplineSurface_Test__WeightsAccess_NonRational() {
    let myOriginalSurface = SetUp();
    let aWeights: Option<Array2<'static, f64>> = myOriginalSurface.weights();
    assert!(aWeights.is_none());
}

#[test]
fn Geom_BSplineSurface_Test__PeriodicSurface_SetUNotPeriodic() {
    let _myOriginalSurface = SetUp();
    // U-periodic surface: degree 2, 4 U-knots of mult 1 -> NbUPoles = 1+2 = 3
    let aPoles = grid(3, 3, |i, j| pnt(third(i).cos(), third(i).sin(), f64::from(j) * 0.5));
    let ((anUKnots, anUMults), (aVKnots, aVMults)) = periodic_knots();

    let aSurf = GeomBSplineSurface::new(
        &aPoles,
        (&anUKnots, &anUMults),
        (&aVKnots, &aVMults),
        (2, 2),
        (true, false),
    )
    .unwrap();
    assert!(aSurf.is_u_periodic());

    let aVal = aSurf.value(0.5, 0.5).unwrap();
    aSurf.set_u_not_periodic().unwrap();
    assert!(!aSurf.is_u_periodic());
    let aVal2 = aSurf.value(0.5, 0.5).unwrap();
    assert!(aVal.is_equal(&aVal2, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__PeriodicSurface_SetVNotPeriodic() {
    let _myOriginalSurface = SetUp();
    // V-periodic surface: degree 2, 4 V-knots of mult 1 -> NbVPoles = 1+2 = 3
    let aPoles = grid(3, 3, |i, j| pnt(f64::from(i) * 0.5, third(j).cos(), third(j).sin()));
    let ((aVKnots, aVMults), (anUKnots, anUMults)) = periodic_knots();

    let aSurf = GeomBSplineSurface::new(
        &aPoles,
        (&anUKnots, &anUMults),
        (&aVKnots, &aVMults),
        (2, 2),
        (false, true),
    )
    .unwrap();
    assert!(aSurf.is_v_periodic());

    let aVal = aSurf.value(0.5, 0.5).unwrap();
    aSurf.set_v_not_periodic().unwrap();
    assert!(!aSurf.is_v_periodic());
    let aVal2 = aSurf.value(0.5, 0.5).unwrap();
    assert!(aVal.is_equal(&aVal2, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__RationalSurface() {
    let _myOriginalSurface = SetUp();
    let aPoles = grid(3, 3, |i, j| pnt(f64::from(i), f64::from(j), 0.0));
    let aWeights = grid(3, 3, |i, j| if i == 2 && j == 2 { 3.0 } else { 1.0 });

    let aSurf = rational_patch(&aPoles, &aWeights);
    assert!(aSurf.is_u_rational());
    assert!(aSurf.is_v_rational());

    let aW: Option<Array2<'static, f64>> = aSurf.weights();
    assert!(aW.is_some());
    EXPECT_DOUBLE_EQ(*aW.unwrap().value(2, 2).unwrap(), 3.0);

    // Evaluate
    let aMid = aSurf.value(0.5, 0.5).unwrap();
    // High weight at center should pull surface toward (2,2,0)
    EXPECT_NEAR(aMid.x(), 2.0, 0.1);
    EXPECT_NEAR(aMid.y(), 2.0, 0.1);
}

#[test]
fn Geom_BSplineSurface_Test__LocalEvaluation() {
    let myOriginalSurface = SetUp();
    myOriginalSurface.insert_u_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalSurface.nb_u_knots(), 3);

    let aGlobal = myOriginalSurface.value(0.25, 0.5).unwrap();
    let aLocal = myOriginalSurface.local_d0(0.25, 0.5, ((1, 2), (1, 2))).unwrap();
    assert!(aGlobal.is_equal(&aLocal, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__SetPoleRow() {
    let myOriginalSurface = SetUp();
    let aNewRow = array(&[pnt(10.0, 1.0, 0.0), pnt(10.0, 2.0, 0.0), pnt(10.0, 3.0, 0.0)]);

    myOriginalSurface.set_pole_row(2, &aNewRow).unwrap();
    for j in 1..=3 {
        EXPECT_NEAR(myOriginalSurface.pole(2, j).unwrap().x(), 10.0, 1e-10);
    }
}

#[test]
fn Geom_BSplineSurface_Test__SetPoleCol() {
    let myOriginalSurface = SetUp();
    let aNewCol = array(&[pnt(1.0, 10.0, 0.0), pnt(2.0, 10.0, 0.0), pnt(3.0, 10.0, 0.0)]);

    myOriginalSurface.set_pole_col(2, &aNewCol).unwrap();
    for i in 1..=3 {
        EXPECT_NEAR(myOriginalSurface.pole(i, 2).unwrap().y(), 10.0, 1e-10);
    }
}

#[test]
fn Geom_BSplineSurface_Test__SetWeightRow() {
    let _myOriginalSurface = SetUp();
    // Create rational surface
    let (aPoles, aWeights) = raised_centre();
    let aSurf = rational_patch(&aPoles, &aWeights);

    let aNewWeights = array(&[3.0, 4.0, 5.0]);

    aSurf.set_weight_row(2, &aNewWeights).unwrap();
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 1).unwrap(), 3.0);
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 2).unwrap(), 4.0);
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 3).unwrap(), 5.0);
}

#[test]
fn Geom_BSplineSurface_Test__SetWeightCol() {
    let _myOriginalSurface = SetUp();
    let (aPoles, aWeights) = raised_centre();
    let aSurf = rational_patch(&aPoles, &aWeights);

    let aNewWeights = array(&[6.0, 7.0, 8.0]);

    aSurf.set_weight_col(2, &aNewWeights).unwrap();
    EXPECT_DOUBLE_EQ(aSurf.weight(1, 2).unwrap(), 6.0);
    EXPECT_DOUBLE_EQ(aSurf.weight(2, 2).unwrap(), 7.0);
    EXPECT_DOUBLE_EQ(aSurf.weight(3, 2).unwrap(), 8.0);
}

#[test]
fn Geom_BSplineSurface_Test__RemoveVKnot() {
    let myOriginalSurface = SetUp();
    myOriginalSurface.insert_v_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalSurface.nb_v_knots(), 3);

    let aValBefore = myOriginalSurface.value(0.5, 0.25).unwrap();
    let isRemoved: bool = myOriginalSurface.remove_v_knot(2, 0, 1e-6).unwrap();
    assert!(isRemoved);
    assert_eq!(myOriginalSurface.nb_v_knots(), 2);
    let aValAfter = myOriginalSurface.value(0.5, 0.25).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-6));
}

#[test]
fn Geom_BSplineSurface_Test__MovePoint() {
    let myOriginalSurface = SetUp();
    let aTarget = pnt(2.0, 2.0, 1.0);
    let aRange = MoveRange {
        u_index1: 1,
        u_index2: myOriginalSurface.nb_u_poles(),
        v_index1: 1,
        v_index2: myOriginalSurface.nb_v_poles(),
    };
    let (_aUFirst, _aULast, _aVFirst, _aVLast) = myOriginalSurface
        .move_point(0.5, 0.5, aTarget, aRange)
        .unwrap();
    let aMoved = myOriginalSurface.value(0.5, 0.5).unwrap();
    assert!(aMoved.is_equal(&aTarget, 1e-6));
}

#[test]
fn Geom_BSplineSurface_Test__LocateU() {
    let myOriginalSurface = SetUp();
    myOriginalSurface.insert_u_knot(0.5, 1, 0.0, true).unwrap();
    let (anI1, anI2) = myOriginalSurface.locate_u(0.25, 1e-10, false).unwrap();
    assert!(anI1 >= 1);
    assert!(anI2 <= myOriginalSurface.nb_u_knots());
}

#[test]
fn Geom_BSplineSurface_Test__LocateV() {
    let myOriginalSurface = SetUp();
    myOriginalSurface.insert_v_knot(0.5, 1, 0.0, true).unwrap();
    let (anI1, anI2) = myOriginalSurface.locate_v(0.25, 1e-10, false).unwrap();
    assert!(anI1 >= 1);
    assert!(anI2 <= myOriginalSurface.nb_v_knots());
}

#[test]
fn Geom_BSplineSurface_Test__RationalSurface_SetUNotPeriodic() {
    let _myOriginalSurface = SetUp();
    // Rational U-periodic surface: exercises the SetUNotPeriodic weight-handling code
    let aPoles = grid(3, 3, |i, j| pnt(third(i).cos(), third(i).sin(), f64::from(j) * 0.5));
    let aWeights = grid(3, 3, |i, _| 1.0 + 0.5 * f64::from(i - 1));
    let ((anUKnots, anUMults), (aVKnots, aVMults)) = periodic_knots();

    let aSurf = GeomBSplineSurface::with_weights(
        &aPoles,
        &aWeights,
        (&anUKnots, &anUMults),
        (&aVKnots, &aVMults),
        (2, 2),
        (true, false),
    )
    .unwrap();
    assert!(aSurf.is_u_periodic());
    assert!(aSurf.is_u_rational() || aSurf.is_v_rational());

    let aVal = aSurf.value(0.5, 0.5).unwrap();
    aSurf.set_u_not_periodic().unwrap();
    assert!(!aSurf.is_u_periodic());
    let aVal2 = aSurf.value(0.5, 0.5).unwrap();
    assert!(aVal.is_equal(&aVal2, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__RationalSurface_SetVNotPeriodic() {
    let _myOriginalSurface = SetUp();
    // Rational V-periodic surface
    let aPoles = grid(3, 3, |i, j| pnt(f64::from(i) * 0.5, third(j).cos(), third(j).sin()));
    let aWeights = grid(3, 3, |_, j| 1.0 + 0.5 * f64::from(j - 1));
    let ((aVKnots, aVMults), (anUKnots, anUMults)) = periodic_knots();

    let aSurf = GeomBSplineSurface::with_weights(
        &aPoles,
        &aWeights,
        (&anUKnots, &anUMults),
        (&aVKnots, &aVMults),
        (2, 2),
        (false, true),
    )
    .unwrap();
    assert!(aSurf.is_v_periodic());
    assert!(aSurf.is_u_rational() || aSurf.is_v_rational());

    let aVal = aSurf.value(0.5, 0.5).unwrap();
    aSurf.set_v_not_periodic().unwrap();
    assert!(!aSurf.is_v_periodic());
    let aVal2 = aSurf.value(0.5, 0.5).unwrap();
    assert!(aVal.is_equal(&aVal2, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__Evaluation_D3() {
    let myOriginalSurface = SetUp();
    let aResult = myOriginalSurface.d3(0.5, 0.5).unwrap();
    // Degree 2 surface: D3 should be zero
    EXPECT_NEAR(aResult.d3u.magnitude(), 0.0, 1e-10);
    EXPECT_NEAR(aResult.d3v.magnitude(), 0.0, 1e-10);
}

#[test]
fn Geom_BSplineSurface_Test__IncreaseUMultiplicity() {
    let myOriginalSurface = SetUp();
    myOriginalSurface.insert_u_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalSurface.u_multiplicity(2).unwrap(), 1);

    let aValBefore = myOriginalSurface.value(0.5, 0.5).unwrap();
    myOriginalSurface.increase_u_multiplicity(2, 2).unwrap();
    assert_eq!(myOriginalSurface.u_multiplicity(2).unwrap(), 2);
    let aValAfter = myOriginalSurface.value(0.5, 0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineSurface_Test__SetUKnot() {
    let myOriginalSurface = SetUp();
    myOriginalSurface.insert_u_knot(0.5, 1, 0.0, true).unwrap();
    myOriginalSurface.set_u_knot(2, 0.6).unwrap();
    EXPECT_DOUBLE_EQ(myOriginalSurface.u_knot(2).unwrap(), 0.6);
}

#[test]
fn Geom_BSplineSurface_Test__SetVKnot() {
    let myOriginalSurface = SetUp();
    myOriginalSurface.insert_v_knot(0.5, 1, 0.0, true).unwrap();
    myOriginalSurface.set_v_knot(2, 0.6).unwrap();
    EXPECT_DOUBLE_EQ(myOriginalSurface.v_knot(2).unwrap(), 0.6);
}

#[test]
fn Geom_BSplineSurface_Test__RationalSurface_UIso() {
    let _myOriginalSurface = SetUp();
    let aPoles = grid(3, 3, |i, j| pnt(f64::from(i), f64::from(j), f64::from(i + j) * 0.1));
    let aWeights = grid(3, 3, |i, j| 1.0 + 0.5 * f64::from((i - 1) + (j - 1)));

    let aSurf = rational_patch(&aPoles, &aWeights);

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
fn Geom_BSplineSurface_Test__CopyIndependence_Knots() {
    let myOriginalSurface = SetUp();
    let aCopy = copied(&myOriginalSurface);
    myOriginalSurface.insert_u_knot(0.5, 1, 0.0, true).unwrap();
    // Original now has 3 U-knots, copy should still have 2
    assert_eq!(aCopy.nb_u_knots(), 2);
    assert_eq!(myOriginalSurface.nb_u_knots(), 3);
}

#[test]
fn Geom_BSplineSurface_Test__WeightsArray_NonRational_ReturnsUnitWeights() {
    let myOriginalSurface = SetUp();
    assert!(!myOriginalSurface.is_u_rational());
    assert!(!myOriginalSurface.is_v_rational());

    let aAddress = myOriginalSurface.weights_array(|aWeights| {
        assert_eq!(aWeights.nb_rows() as i32, myOriginalSurface.nb_u_poles());
        assert_eq!(aWeights.nb_columns() as i32, myOriginalSurface.nb_v_poles());
        assert!(!aWeights.is_deletable());
        for i in aWeights.lower_row()..=aWeights.upper_row() {
            for j in aWeights.lower_col()..=aWeights.upper_col() {
                EXPECT_DOUBLE_EQ(*aWeights.value(i, j).unwrap(), 1.0);
            }
        }
        std::ptr::from_ref(aWeights)
    });
    assert_eq!(aAddress, myOriginalSurface.weights_array(std::ptr::from_ref));
}

#[test]
fn Geom_BSplineSurface_Test__WeightsArray_Rational_ReturnsOwning() {
    let _myOriginalSurface = SetUp();
    let aPoles = grid(3, 3, |i, j| pnt(f64::from(i), f64::from(j), 0.0));
    let aWeightsIn = grid(3, 3, |i, j| if i == 2 && j == 2 { 2.0 } else { 1.0 });

    let aRational = rational_patch(&aPoles, &aWeightsIn);

    let aAddress = aRational.weights_array(|aWeights| {
        assert_eq!(aWeights.nb_rows(), 3);
        assert_eq!(aWeights.nb_columns(), 3);
        assert!(aWeights.is_deletable());
        EXPECT_DOUBLE_EQ(*aWeights.value(2, 2).unwrap(), 2.0);
        EXPECT_DOUBLE_EQ(*aWeights.value(1, 1).unwrap(), 1.0);
        std::ptr::from_ref(aWeights)
    });
    assert_eq!(aAddress, aRational.weights_array(std::ptr::from_ref));
}

// OCC30990: Foundation Classes - unexpected change in numerical results on bsplines after 0029769
// Verify that evaluating a B-Spline surface at a knot gives consistent results regardless of
// which span was evaluated beforehand (i.e., the cache is updated correctly).
#[test]
fn Geom_BSplineSurface_Test__OCC30990_CacheConsistencyAtKnots() {
    let _myOriginalSurface = SetUp();
    // Build a degree-3 B-Spline surface with 3 interior knots in U (4 spans) and 2 in V (3 spans).
    // Poles: 7 x 5
    let aNbU: i32 = 7;
    let aNbV: i32 = 5;
    let aPoles = grid(aNbU, aNbV, |i, j| {
        pnt(
            f64::from(i - 1),
            f64::from(j - 1),
            (f64::from(i) * 0.5).sin() * (f64::from(j) * 0.7).cos(),
        )
    });

    // Knot vector in U: [0, 0.25, 0.5, 0.75, 1] with multiplicities [4, 1, 1, 1, 4]
    let aUKnots = array(&[0.0, 0.25, 0.5, 0.75, 1.0]);
    let aUMults = array(&[4, 1, 1, 1, 4]);

    // Knot vector in V: [0, 0.5, 1] with multiplicities [4, 1, 4]
    let aVKnots = array(&[0.0, 0.5, 1.0]);
    let aVMults = array(&[4, 1, 4]);

    let aSurf: Handle<GeomBSplineSurface> = Handle::new(
        GeomBSplineSurface::new(
            &aPoles,
            (&aUKnots, &aUMults),
            (&aVKnots, &aVMults),
            (3, 3),
            (false, false),
        )
        .unwrap(),
    );
    assert!(!aSurf.is_null());

    let aAdaptor = GeomAdaptorSurface::with_surface(&aSurf.clone().cast()).unwrap();

    // For each interior U knot, verify that evaluation at the knot is consistent
    // regardless of whether the previous evaluation was in the span before or after.
    let mut aNbErr: i32 = 0;
    for i in 2..aSurf.nb_u_knots() {
        let aUknot: f64 = aSurf.u_knot(i).unwrap();
        let aUprev: f64 = 0.5 * (aUknot + aSurf.u_knot(i - 1).unwrap());
        let aUnext: f64 = 0.5 * (aUknot + aSurf.u_knot(i + 1).unwrap());

        for j in 1..aSurf.nb_v_knots() {
            let aV: f64 = 0.5 * (aSurf.v_knot(j).unwrap() + aSurf.v_knot(j + 1).unwrap());
            aAdaptor.value(aUprev, aV).unwrap(); // populate cache from span before
            let aP1 = aAdaptor.value(aUknot, aV).unwrap();
            aAdaptor.value(aUnext, aV).unwrap(); // populate cache from span after
            let aP2 = aAdaptor.value(aUknot, aV).unwrap();

            if aP1.x() != aP2.x() || aP1.y() != aP2.y() || aP1.z() != aP2.z() {
                aNbErr += 1;
            }
        }
    }

    // Same check for interior V knots
    for j in 2..aSurf.nb_v_knots() {
        let aVknot: f64 = aSurf.v_knot(j).unwrap();
        let aVprev: f64 = 0.5 * (aVknot + aSurf.v_knot(j - 1).unwrap());
        let aVnext: f64 = 0.5 * (aVknot + aSurf.v_knot(j + 1).unwrap());

        for i in 1..aSurf.nb_u_knots() {
            let aU: f64 = 0.5 * (aSurf.u_knot(i).unwrap() + aSurf.u_knot(i + 1).unwrap());
            aAdaptor.value(aU, aVprev).unwrap();
            let aP1 = aAdaptor.value(aU, aVknot).unwrap();
            aAdaptor.value(aU, aVnext).unwrap();
            let aP2 = aAdaptor.value(aU, aVknot).unwrap();

            if aP1.x() != aP2.x() || aP1.y() != aP2.y() || aP1.z() != aP2.z() {
                aNbErr += 1;
            }
        }
    }

    assert_eq!(
        0, aNbErr,
        "BSpline surface cache is inconsistent at span knots"
    );
}
