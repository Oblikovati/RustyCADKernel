// Migrated from tests/lowalgos/extcc/bug29712_34.  The bounded concentric
// arcs intentionally exercise the asymmetric extrema enumeration order.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_34_AsymmetricConcentricArcs() {
    let aCurve1 = makeOCC29712TrimmedCircle(100.0, 0.0, PI);
    let aCurve2 = makeOCC29712TrimmedCircle(50.0, -2.0, 0.0);

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aCurve1, &aCurve2).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema12.nb_extrema(), 2);
    let aDistance121 = anExtrema12.distance(1).unwrap();
    let aDistance122 = anExtrema12.distance(2).unwrap();
    assert_near(aDistance121.min(aDistance122), 50.0, 1.0e-7);
    assert_near(aDistance121.max(aDistance122), 150.0, 1.0e-7);

    let anExtrema21 = ExtremaCurveCurve::with_curves(&aCurve2, &aCurve1).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema21.nb_extrema(), 1);
    assert_near(anExtrema21.distance(1).unwrap(), 50.0, 1.0e-7);
}

// Migrated from tests/lowalgos/extcc/bug29712_35.  Both argument orders
// produce the two finite radial extrema for these opposite half-arcs.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_35_OppositeConcentricArcs() {
    let aCurve1 = makeOCC29712TrimmedCircle(100.0, 0.0, PI);
    let aCurve2 = makeOCC29712TrimmedCircle(50.0, PI, 2.0 * PI);

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aCurve1, &aCurve2).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema12.nb_extrema(), 2);
    let aDistance121 = anExtrema12.distance(1).unwrap();
    let aDistance122 = anExtrema12.distance(2).unwrap();
    assert_near(aDistance121.min(aDistance122), 50.0, 1.0e-7);
    assert_near(aDistance121.max(aDistance122), 150.0, 1.0e-7);

    let anExtrema21 = ExtremaCurveCurve::with_curves(&aCurve2, &aCurve1).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema21.nb_extrema(), 2);
    let aDistance211 = anExtrema21.distance(1).unwrap();
    let aDistance212 = anExtrema21.distance(2).unwrap();
    assert_near(aDistance211.min(aDistance212), 50.0, 1.0e-7);
    assert_near(aDistance211.max(aDistance212), 150.0, 1.0e-7);
}

// Migrated from tests/lowalgos/extcc/bug29712_36.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_36_ParallelFullCircleAndArc() {
    checkOCC29712ConcentricCircles(2.0 * PI, -12.0, -10.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_37.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_37_SeparatedConcentricArcs() {
    let aCurve1 = makeOCC29712MirroredTrimmedCircle(100.0, 3.0, 6.5);
    let aCurve2 = makeOCC29712TrimmedCircle(50.0, 4.0, 6.0);
    let anExtrema12 = ExtremaCurveCurve::with_curves(&aCurve1, &aCurve2).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert!(anExtrema12.nb_extrema() > 0);
    let anExtrema21 = ExtremaCurveCurve::with_curves(&aCurve2, &aCurve1).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert!(anExtrema21.nb_extrema() > 0);
}

// Migrated from tests/lowalgos/extcc/bug29712_38.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_38_ParallelConcentricArcs() {
    let aCurve1 = makeOCC29712MirroredTrimmedCircle(100.0, 0.0, 4.0);
    let aCurve2 = makeOCC29712TrimmedCircle(50.0, 4.0, 6.0);
    checkParallelExtrema(&aCurve1, &aCurve2, 50.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_39.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_39_CircleAndAxisSegment() {
    let aCircle = makeOCC29712TrimmedCircle(5.0, 0.0, 2.0 * PI);
    let aSegment = trimmed(&line(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)), -1.0, 1.0);
    checkParallelExtrema(&aCircle, &aSegment, 5.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_41.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_41_CircleAndNearAxisSegment() {
    let aCircle = makeOCC29712TrimmedCircle(5.0, 0.0, 2.0 * PI);
    let aSegment = trimmed(&line(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)), 3.0e-8, 1.0);
    checkParallelExtrema(&aCircle, &aSegment, 5.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_42.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_42_CircleAndTooNearAxisSegment() {
    let aCircle = makeOCC29712TrimmedCircle(5.0, 0.0, 2.0 * PI);
    let aSegment = trimmed(&line(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)), 2.0e-7, 1.0);

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aCircle, &aSegment).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema12.nb_extrema(), 0);
    let anExtrema21 = ExtremaCurveCurve::with_curves(&aSegment, &aCircle).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema21.nb_extrema(), 0);
}
