// Geom_BSplineCurve_Test.cxx, tests PeriodicCurve to WeightsArray_Rational_ReturnsOwning.

#[test]
fn Geom_BSplineCurve_Test__PeriodicCurve() {
    let _myOriginalCurve = SetUp();
    // Periodic BSpline: NbPoles = Sum(Mults) - Mults(last)
    // 5 poles, degree 3, 6 knots of mult 1 -> NbPoles = 1+4 = 5
    let aCurve = periodic_pentagon();
    assert!(aCurve.is_periodic());

    let aVal1 = aCurve.value(0.5).unwrap();
    aCurve.set_not_periodic().unwrap();
    assert!(!aCurve.is_periodic());

    let aVal2 = aCurve.value(0.5).unwrap();
    assert!(aVal1.is_equal(&aVal2, 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__MultiKnotSpan() {
    let _myOriginalCurve = SetUp();
    // BSpline with 3 knot spans, degree 2
    // NbPoles = Mf + Ml - (Deg+1) + Sum(interior) = 3+3-3+1+1 = 5
    let aPoles = array(&[
        pnt(0.0, 0.0, 0.0),
        pnt(1.0, 2.0, 0.0),
        pnt(2.0, 0.0, 0.0),
        pnt(3.0, 2.0, 0.0),
        pnt(4.0, 0.0, 0.0),
    ]);
    let aKnots = array(&[0.0, 1.0, 2.0, 3.0]);
    let aMults = array(&[3, 1, 1, 3]);

    let aCurve = Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap());
    assert_eq!(aCurve.nb_knots(), 4);
    assert_eq!(aCurve.nb_poles(), 5);

    // Evaluate at knot boundaries
    let aPnt = aCurve.d0(0.0).unwrap();
    assert!(aPnt.is_equal(&pnt(0.0, 0.0, 0.0), 1e-10));
    let aPnt = aCurve.d0(3.0).unwrap();
    assert!(aPnt.is_equal(&pnt(4.0, 0.0, 0.0), 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__IncreaseMultiplicity() {
    let myOriginalCurve = SetUp();
    // Add a knot first to have an interior knot
    myOriginalCurve.insert_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalCurve.nb_knots(), 3);
    assert_eq!(myOriginalCurve.multiplicity(2).unwrap(), 1);

    let aValBefore = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve.increase_multiplicity(2, 2).unwrap();
    assert_eq!(myOriginalCurve.multiplicity(2).unwrap(), 2);
    let aValAfter = myOriginalCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__KnotAccess() {
    let myOriginalCurve = SetUp();
    EXPECT_DOUBLE_EQ(myOriginalCurve.knot(1).unwrap(), 0.0);
    EXPECT_DOUBLE_EQ(myOriginalCurve.knot(2).unwrap(), 1.0);
    assert_eq!(myOriginalCurve.first_u_knot_index().unwrap(), 1);
    assert_eq!(myOriginalCurve.last_u_knot_index().unwrap(), 2);

    let aKnots: Array1<'static, f64> = myOriginalCurve.knots();
    assert_eq!(aKnots.len(), 2);

    let aFlatKnots: Array1<'static, f64> = myOriginalCurve.knot_sequence();
    assert_eq!(aFlatKnots.len(), 8);

    let aMults: Array1<'static, i32> = myOriginalCurve.multiplicities();
    assert_eq!(aMults.len(), 2);
    assert_eq!(aMults[1], 4);
    assert_eq!(aMults[2], 4);
}

#[test]
fn Geom_BSplineCurve_Test__PolesAccess() {
    let myOriginalCurve = SetUp();
    let aPoles: Array1<'static, Point3> = myOriginalCurve.poles();
    assert_eq!(aPoles.len(), 4);
    assert!(aPoles[1].is_equal(&pnt(0.0, 0.0, 0.0), 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__WeightsAccess_NonRational() {
    let myOriginalCurve = SetUp();
    let aWeights: Option<Array1<'static, f64>> = myOriginalCurve.weights();
    assert!(aWeights.is_none());
}

#[test]
fn Geom_BSplineCurve_Test__WeightsAccess_Rational() {
    let _myOriginalCurve = SetUp();
    let aCurve = rational_quadratic(
        [pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0)],
        2.0,
    );
    let aW: Option<Array1<'static, f64>> = aCurve.weights();
    assert!(aW.is_some());
    EXPECT_DOUBLE_EQ(aW.unwrap()[2], 2.0);
}

#[test]
fn Geom_BSplineCurve_Test__LocalEvaluation() {
    let myOriginalCurve = SetUp();
    // Insert a knot to create 2 spans
    myOriginalCurve.insert_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(myOriginalCurve.nb_knots(), 3);

    let aGlobal = myOriginalCurve.value(0.25).unwrap();
    let aLocal = myOriginalCurve.local_value(0.25, 1, 2).unwrap();
    assert!(aGlobal.is_equal(&aLocal, 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__SetKnot() {
    let _myOriginalCurve = SetUp();
    // Create a curve with 3 knots, degree 2
    // NbPoles = 3+3-3+1 = 4
    let aPoles = array(&[
        pnt(0.0, 0.0, 0.0),
        pnt(1.0, 1.0, 0.0),
        pnt(2.0, 0.0, 0.0),
        pnt(3.0, 1.0, 0.0),
    ]);
    let aKnots = array(&[0.0, 0.5, 1.0]);
    let aMults = array(&[3, 1, 3]);

    let aCurve = Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap());

    aCurve.set_knot(2, 0.6).unwrap();
    EXPECT_DOUBLE_EQ(aCurve.knot(2).unwrap(), 0.6);
}

#[test]
fn Geom_BSplineCurve_Test__CopyIndependence() {
    let myOriginalCurve = SetUp();
    let aCopy = copied(&myOriginalCurve);
    myOriginalCurve.set_pole(2, pnt(10.0, 10.0, 10.0)).unwrap();
    assert!(
        !aCopy
            .pole(2)
            .unwrap()
            .is_equal(&pnt(10.0, 10.0, 10.0), 1e-10)
    );
}

#[test]
fn Geom_BSplineCurve_Test__MovePoint() {
    let myOriginalCurve = SetUp();
    let aTarget = pnt(1.5, 0.8, 0.0);
    let (aFirst, _aLast) = myOriginalCurve
        .move_point(0.5, aTarget, 1, myOriginalCurve.nb_poles())
        .unwrap();
    assert!(aFirst > 0);
    let aMoved = myOriginalCurve.value(0.5).unwrap();
    assert!(aMoved.is_equal(&aTarget, 1e-6));
}

#[test]
fn Geom_BSplineCurve_Test__LocateU() {
    let myOriginalCurve = SetUp();
    myOriginalCurve.insert_knot(0.5, 1, 0.0, true).unwrap();
    let (anI1, anI2) = myOriginalCurve.locate_u(0.25, 1e-10, false).unwrap();
    assert!(anI1 >= 1);
    assert!(anI2 <= myOriginalCurve.nb_knots());
    assert!(myOriginalCurve.knot(anI1).unwrap() <= 0.25 + 1e-10);
    assert!(myOriginalCurve.knot(anI2).unwrap() >= 0.25 - 1e-10);
}

#[test]
fn Geom_BSplineCurve_Test__IsEqual() {
    let myOriginalCurve = SetUp();
    let aCopy = copied(&myOriginalCurve);
    assert!(myOriginalCurve.is_equal(&aCopy, 1e-10).unwrap());

    aCopy.set_pole(2, pnt(10.0, 10.0, 10.0)).unwrap();
    assert!(!myOriginalCurve.is_equal(&aCopy, 1e-10).unwrap());
}

#[test]
fn Geom_BSplineCurve_Test__KnotDistribution() {
    let myOriginalCurve = SetUp();
    let aDistr: BSplKnotDistribution = myOriginalCurve.knot_distribution();
    // Single span with end mults = degree+1 -> PiecewiseBezier
    assert_eq!(aDistr, BSplKnotDistribution::PiecewiseBezier);
}

#[test]
fn Geom_BSplineCurve_Test__RationalCurveSegment() {
    let _myOriginalCurve = SetUp();
    let aCurve = rational_quadratic(
        [pnt(1.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(0.0, 1.0, 0.0)],
        1.0 / 2.0_f64.sqrt(),
    );
    assert!(aCurve.is_rational());

    let aMid = aCurve.value(0.5).unwrap();
    aCurve.segment(0.25, 0.75, precision::p_confusion()).unwrap();
    let aMidAfter = aCurve.value(0.5).unwrap();
    assert!(aMid.is_equal(&aMidAfter, 1e-6));
    assert!(aCurve.is_rational());
}

#[test]
fn Geom_BSplineCurve_Test__RationalCurveIncreaseDegree() {
    let _myOriginalCurve = SetUp();
    let aCurve = rational_quadratic(
        [pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0)],
        2.0,
    );
    let aValBefore = aCurve.value(0.5).unwrap();

    aCurve.increase_degree(4).unwrap();
    assert_eq!(aCurve.degree(), 4);
    assert!(aCurve.is_rational());
    let aValAfter = aCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__PeriodicCurve_SetOrigin() {
    let _myOriginalCurve = SetUp();
    let aCurve = periodic_pentagon();
    assert!(aCurve.is_periodic());

    aCurve.set_origin(3).unwrap();
    assert!(aCurve.is_periodic());

    // Curve shape unchanged, just reparameterized -- can evaluate without crash
    let aVal2 = aCurve.value(aCurve.first_parameter()).unwrap();
    assert!(aVal2.x() * aVal2.x() + aVal2.y() * aVal2.y() > 0.0);
}

#[test]
fn Geom_BSplineCurve_Test__InsertKnots_Multiple() {
    let myOriginalCurve = SetUp();
    let aNewKnots = array(&[0.25, 0.75]);
    let aNewMults = array(&[1, 1]);

    let aValBefore = myOriginalCurve.value(0.5).unwrap();
    myOriginalCurve
        .insert_knots(&aNewKnots, &aNewMults, 0.0, false)
        .unwrap();
    assert_eq!(myOriginalCurve.nb_knots(), 4);
    let aValAfter = myOriginalCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BSplineCurve_Test__ClosedCurve() {
    let _myOriginalCurve = SetUp();
    // Create a closed BSpline (first pole == last pole)
    let aPoles = array(&[
        pnt(0.0, 0.0, 0.0),
        pnt(1.0, 1.0, 0.0),
        pnt(2.0, 0.0, 0.0),
        pnt(0.0, 0.0, 0.0),
    ]);
    let aKnots = array(&[0.0, 1.0]);
    let aMults = array(&[4, 4]);

    let aCurve = Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap());
    assert!(aCurve.is_closed().unwrap());
}

#[test]
fn Geom_BSplineCurve_Test__LocalD1() {
    let myOriginalCurve = SetUp();
    myOriginalCurve.insert_knot(0.5, 1, 0.0, true).unwrap();
    let (aPnt, aV1) = myOriginalCurve.d1(0.25).unwrap();
    let (aPntL, aV1L) = myOriginalCurve.local_d1(0.25, 1, 2).unwrap();
    assert!(aPnt.is_equal(&aPntL, 1e-10));
    EXPECT_NEAR(aV1.x(), aV1L.x(), 1e-10);
    EXPECT_NEAR(aV1.y(), aV1L.y(), 1e-10);
}

#[test]
fn Geom_BSplineCurve_Test__WeightsArray_NonRational_ReturnsUnitWeights() {
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
    // Verify that the reference is stable
    assert_eq!(aAddress, myOriginalCurve.weights_array(std::ptr::from_ref));
}

#[test]
fn Geom_BSplineCurve_Test__WeightsArray_Rational_ReturnsOwning() {
    let _myOriginalCurve = SetUp();
    // Create a rational BSpline
    let aPoles = array(&[
        pnt(0.0, 0.0, 0.0),
        pnt(1.0, 1.0, 0.0),
        pnt(2.0, 1.0, 0.0),
        pnt(3.0, 0.0, 0.0),
    ]);
    let aWeightsIn = array(&[1.0, 2.0, 3.0, 1.0]);
    let aKnots = array(&[0.0, 1.0]);
    let aMults = array(&[4, 4]);

    let aRational = Handle::new(
        GeomBSplineCurve::with_weights(&aPoles, &aWeightsIn, &aKnots, &aMults, 3, false).unwrap(),
    );
    assert!(aRational.is_rational());

    let aAddress = aRational.weights_array(|aWeights| {
        assert_eq!(aWeights.len(), 4);
        assert!(aWeights.is_deletable());
        EXPECT_DOUBLE_EQ(aWeights[1], 1.0);
        EXPECT_DOUBLE_EQ(aWeights[2], 2.0);
        EXPECT_DOUBLE_EQ(aWeights[3], 3.0);
        EXPECT_DOUBLE_EQ(aWeights[4], 1.0);
        std::ptr::from_ref(aWeights)
    });
    // Verify that the reference is stable
    assert_eq!(aAddress, aRational.weights_array(std::ptr::from_ref));
}
