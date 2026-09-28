// Geom_BezierCurve_Test.cxx, tests PolesAccess to OCC2569_ThrowsForTooManyPoles.

#[test]
fn Geom_BezierCurve_Test__PolesAccess() {
    let myOriginalCurve = SetUp();
    let aPoles: Array1<'static, Point3> = myOriginalCurve.poles();
    assert_eq!(aPoles.len(), 4);
    assert!(aPoles[1].is_equal(&pnt(0.0, 0.0, 0.0), 1e-10));
}

#[test]
fn Geom_BezierCurve_Test__WeightsAccess_NonRational() {
    let myOriginalCurve = SetUp();
    let aWeights: Option<Array1<'static, f64>> = myOriginalCurve.weights();
    assert!(aWeights.is_none());
}

#[test]
fn Geom_BezierCurve_Test__RationalCurveEvaluation() {
    let _myOriginalCurve = SetUp();
    // Create a rational quadratic Bezier (approximation of circular arc)
    let aPoles = poles3(pnt(1.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(0.0, 1.0, 0.0));
    let aWeights = weights3(1.0, 1.0 / 2.0_f64.sqrt(), 1.0);

    let aCurve = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    assert!(aCurve.is_rational());

    // At u=0.5, the rational curve should be close to the unit circle
    let aMid = aCurve.value(0.5).unwrap();
    let aRadius: f64 = (aMid.x() * aMid.x() + aMid.y() * aMid.y()).sqrt();
    EXPECT_NEAR(aRadius, 1.0, 1e-6);
}

#[test]
fn Geom_BezierCurve_Test__MaxDegree() {
    let _myOriginalCurve = SetUp();
    assert!(GeomBezierCurve::max_degree() >= 25);
}

#[test]
fn Geom_BezierCurve_Test__SetPoleWithWeight() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0));
    let aWeights = weights3(1.0, 1.0, 1.0);

    let aCurve = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aNewPole = pnt(1.0, 2.0, 0.0);
    aCurve.set_pole_weighted(2, aNewPole, 5.0).unwrap();
    assert!(aCurve.pole(2).unwrap().is_equal(&aNewPole, 1e-10));
    EXPECT_DOUBLE_EQ(aCurve.weight(2).unwrap(), 5.0);
    assert!(aCurve.is_rational());
}

#[test]
fn Geom_BezierCurve_Test__InsertPoleAfterWithWeight() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0));
    let aWeights = weights3(1.0, 2.0, 1.0);

    let aCurve = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    aCurve
        .insert_pole_after_weighted(2, pnt(1.5, 0.5, 0.0), 3.0)
        .unwrap();
    assert_eq!(aCurve.nb_poles(), 4);
    EXPECT_DOUBLE_EQ(aCurve.weight(3).unwrap(), 3.0);
    assert!(aCurve.is_rational());
}

#[test]
fn Geom_BezierCurve_Test__ClosedCurve() {
    let _myOriginalCurve = SetUp();
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = pnt(0.0, 0.0, 0.0);
    aPoles[2] = pnt(1.0, 1.0, 0.0);
    aPoles[3] = pnt(2.0, 0.0, 0.0);
    aPoles[4] = pnt(0.0, 0.0, 0.0);

    let aCurve = Handle::new(GeomBezierCurve::new(&aPoles).unwrap());
    assert!(aCurve.is_closed().unwrap());
}

#[test]
fn Geom_BezierCurve_Test__RationalSegment() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(pnt(1.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(0.0, 1.0, 0.0));
    let aWeights = weights3(1.0, 1.0 / 2.0_f64.sqrt(), 1.0);

    let aCurve = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aMid = aCurve.value(0.5).unwrap();

    aCurve.segment(0.25, 0.75).unwrap();
    let aMidAfter = aCurve.value(0.5).unwrap();
    assert!(aMid.is_equal(&aMidAfter, 1e-6));
    assert!(aCurve.is_rational());
}

#[test]
fn Geom_BezierCurve_Test__RationalIncrease() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0));
    let aWeights = weights3(1.0, 2.0, 1.0);

    let aCurve = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aValBefore = aCurve.value(0.5).unwrap();

    aCurve.increase(5).unwrap();
    assert_eq!(aCurve.degree(), 5);
    assert!(aCurve.is_rational());
    let aValAfter = aCurve.value(0.5).unwrap();
    assert!(aValBefore.is_equal(&aValAfter, 1e-10));
}

#[test]
fn Geom_BezierCurve_Test__RationalReverse() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0));
    let aWeights = weights3(1.0, 3.0, 2.0);

    let aCurve = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let aStart = aCurve.start_point().unwrap();
    let anEnd = aCurve.end_point().unwrap();

    aCurve.reverse().unwrap();
    assert!(aCurve.start_point().unwrap().is_equal(&anEnd, 1e-10));
    assert!(aCurve.end_point().unwrap().is_equal(&aStart, 1e-10));
    // Weights should be reversed
    EXPECT_DOUBLE_EQ(aCurve.weight(1).unwrap(), 2.0);
    EXPECT_DOUBLE_EQ(aCurve.weight(3).unwrap(), 1.0);
}

#[test]
fn Geom_BezierCurve_Test__ReversedParameter() {
    let myOriginalCurve = SetUp();
    EXPECT_DOUBLE_EQ(myOriginalCurve.reversed_parameter(0.3).unwrap(), 0.7);
    EXPECT_DOUBLE_EQ(myOriginalCurve.reversed_parameter(0.0).unwrap(), 1.0);
}

#[test]
fn Geom_BezierCurve_Test__LinearCurve() {
    let _myOriginalCurve = SetUp();
    // Degree 1 Bezier = line segment
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 2).unwrap();
    aPoles[1] = pnt(0.0, 0.0, 0.0);
    aPoles[2] = pnt(3.0, 4.0, 0.0);

    let aCurve = Handle::new(GeomBezierCurve::new(&aPoles).unwrap());
    assert_eq!(aCurve.degree(), 1);

    let aMid = aCurve.value(0.5).unwrap();
    assert!(aMid.is_equal(&pnt(1.5, 2.0, 0.0), 1e-10));

    // D2 should be zero for a line
    let (_aPnt, _aV1, aV2) = aCurve.d2(0.5).unwrap();
    EXPECT_NEAR(aV2.magnitude(), 0.0, 1e-10);
}

#[test]
fn Geom_BezierCurve_Test__WeightsArray_NonRational_ReturnsUnitWeights() {
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
    // The reference is stable.
    assert_eq!(aAddress, myOriginalCurve.weights_array(std::ptr::from_ref));
}

#[test]
fn Geom_BezierCurve_Test__WeightsArray_Rational_ReturnsOwning() {
    let _myOriginalCurve = SetUp();
    let aPoles = poles3(pnt(0.0, 0.0, 0.0), pnt(1.0, 1.0, 0.0), pnt(2.0, 0.0, 0.0));
    let aWeightsIn = weights3(1.0, 2.0, 1.0);

    let aRational = Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeightsIn).unwrap());
    assert!(aRational.is_rational());

    let aAddress = aRational.weights_array(|aWeights| {
        assert_eq!(aWeights.len(), 3);
        assert!(aWeights.is_deletable());
        EXPECT_DOUBLE_EQ(aWeights[1], 1.0);
        EXPECT_DOUBLE_EQ(aWeights[2], 2.0);
        EXPECT_DOUBLE_EQ(aWeights[3], 1.0);
        std::ptr::from_ref(aWeights)
    });
    // The reference is stable.
    assert_eq!(aAddress, aRational.weights_array(std::ptr::from_ref));
}

// Test OCC2569: Geom_BezierCurve degree equals NbPoles - 1.
// Migrated from QABugs_17.cxx OCC2569
#[test]
fn Geom_BezierCurveTest__OCC2569_DegreeEqualsNbPolesMinusOne() {
    let aNbPoles: i32 = 26;
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, aNbPoles).unwrap();
    for i in 1..=aNbPoles {
        let x = f64::from(i);
        aPoles[i] = pnt(x + 10.0, x * 2.0 + 20.0, x * 3.0 + 45.0);
    }

    let aCurve = Handle::new(GeomBezierCurve::new(&aPoles).unwrap());
    assert!(!aCurve.is_null());
    assert_eq!(aCurve.degree(), aNbPoles - 1);
}

// Test OCC2569: Geom_BezierCurve throws when NbPoles exceeds maximum allowed.
// Migrated from QABugs_17.cxx OCC2569 (bug2569_2)
#[test]
fn Geom_BezierCurveTest__OCC2569_ThrowsForTooManyPoles() {
    let aNbPoles: i32 = 29;
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, aNbPoles).unwrap();
    for i in 1..=aNbPoles {
        let x = f64::from(i);
        aPoles[i] = pnt(x + 10.0, x * 2.0 + 20.0, x * 3.0 + 45.0);
    }

    assert!(GeomBezierCurve::new(&aPoles).is_err());
}
