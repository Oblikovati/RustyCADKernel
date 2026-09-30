// Test OCC862: GeomAPI_ExtremaCurveCurve - Extrema between BSpline and line
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC862_ExtremaBSplineAndLine() {
    // Define BSpline curve data (from Glob_Poles, Glob_Knots, Glob_Mults)
    let aNbPoles: i32 = 195;
    let aNbKnots: i32 = 17;

    // Fill array of poles
    let mut aPolesArray: Array1<'static, Point3> = Array1::with_bounds(1, aNbPoles).unwrap();
    for (i, pole) in OCC862_POLES.iter().enumerate() {
        aPolesArray[i as i32 + 1] = Point3::new(pole[0], pole[1], pole[2]);
    }

    // Fill array of knots
    let mut aKnotsArray: Array1<'static, f64> = Array1::with_bounds(1, aNbKnots).unwrap();
    for (i, knot) in OCC862_KNOTS.iter().enumerate() {
        aKnotsArray[i as i32 + 1] = *knot;
    }

    // Fill array of mults
    let mut aMultsArray: Array1<'static, i32> = Array1::with_bounds(1, aNbKnots).unwrap();
    for (i, mult) in OCC862_MULTS.iter().enumerate() {
        aMultsArray[i as i32 + 1] = *mult;
    }

    // Create B-Spline curve
    let aDegree: i32 = 14;
    let aC1: Handle<dyn GeomCurve> = Handle::new(
        GeomBSplineCurve::new(&aPolesArray, &aKnotsArray, &aMultsArray, aDegree, false).unwrap(),
    )
    .cast();

    // Create trimmed line
    let aP1 = Xyz::new(60000.0, -7504.83, 6000.0);
    let aP2 = Xyz::new(60000.0, 7504.83, 6000.0);
    let aDelta = aP2 - aP1;
    let aLine = line(
        Point3::new(aP1.x(), aP1.y(), aP1.z()),
        dir(aDelta.x(), aDelta.y(), aDelta.z()),
    );
    let aC2 = trimmed(&aLine, 0.0, aDelta.modulus());

    // Try to find extrema
    // IMPORTANT: it is not allowed to input infinite curves!
    let anEx = ExtremaCurveCurve::with_curves(&aC1, &aC2).unwrap();

    // Check if curves are parallel
    if anEx.extrema().unwrap().is_parallel().unwrap() {
        // Parallel case - should have infinite number of extrema
        assert!(anEx.lower_distance().unwrap() > 0.0);
    } else {
        // Check if extrema were found
        let aNbEx = anEx.nb_extrema();
        assert!(aNbEx > 0);

        if aNbEx > 0 {
            // Get minimal distance data
            let (aPoint1, aPoint2) = anEx.nearest_points().unwrap();

            let (aU1, aU2) = anEx.lower_distance_parameters().unwrap();

            let aDistance = anEx.lower_distance().unwrap();

            // Verify minimal distance is reasonable
            assert!(aDistance >= 0.0);

            // Verify parameters are within valid range [0, 1] for the curves
            assert!(aU1 >= 0.0);
            assert!(aU1 <= 1.0);
            assert!(aU2 >= 0.0);

            // Verify nearest points are valid
            let origin = Point3::new(0.0, 0.0, 0.0);
            assert!(!(aPoint1.is_equal(&origin, 1e-10) && aPoint2.is_equal(&origin, 1e-10)));
        }
    }
}

// Migrated from tests/lowalgos/extcc/bug26269_1.  The finite trimming is
// essential: the corresponding infinite lines have infinitely many extrema.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC26269_1_TrimmedParallelLines() {
    let aLine1 = line(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aLine2 = line(Point3::new(2.0, 2.0, 0.0), dir(1.0, 0.0, 0.0));
    let aTrimmed1 = trimmed(&aLine1, 0.0, 1.0);
    let aTrimmed2 = trimmed(&aLine2, 0.0, 1.0);

    let anExtrema = ExtremaCurveCurve::with_curves(&aTrimmed1, &aTrimmed2).unwrap();
    assert!(!anExtrema.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema.nb_extrema(), 1);
    assert_near(anExtrema.distance(1).unwrap(), 5.0_f64.sqrt(), 1.0e-12);
}

// Migrated from tests/lowalgos/extcc/bug26269_2.  Restrict the two concentric
// circles to disjoint arcs and verify that the finite extrema are computed.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC26269_2_TrimmedConcentricArcs() {
    let origin = Point3::new(0.0, 0.0, 0.0);
    let aCircle1 = circle(origin, dir(0.0, 0.0, 1.0), 10.0).cast::<dyn GeomCurve>();
    let aCircle2 = circle(origin, dir(0.0, 0.0, 1.0), 5.0).cast::<dyn GeomCurve>();
    let aTrimmed1 = trimmed(&aCircle1, 0.0, 1.0);
    let aTrimmed2 = trimmed(&aCircle2, 3.0, 4.0);

    let anExtrema = ExtremaCurveCurve::with_curves(&aTrimmed1, &aTrimmed2).unwrap();
    assert!(!anExtrema.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema.nb_extrema(), 1);
    assert!(anExtrema.distance(1).unwrap() > 0.0);
}

// Migrated from tests/lowalgos/extcs/bug29426_1.  The source DRAW test
// accepts an explicit intersection-point report as well as a zero extremum.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29426_1_CircleSphereMinimumDistance() {
    checkOCC29426CircleSphere(11.6635040382361, -3.23970209548358, 9.876);
}

// Migrated from tests/lowalgos/extcs/bug29426_2.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29426_2_CircleSphereMinimumDistance() {
    checkOCC29426CircleSphere(11.5073628633636, -3.18494573739955, 9.712);
}

// Migrated from tests/lowalgos/extcc/bug29712_2.  Bounded parallel B-splines
// must be treated as finite curves rather than as an infinite parallel pair.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_2_BoundedParallelBSplines() {
    let aCurve1 = makeLinearBSpline(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let aCurve2 = makeLinearBSpline(Point3::new(3.0, 4.0, 0.0), Point3::new(4.0, 4.0, 0.0));

    let anExtrema = ExtremaCurveCurve::with_curves(&aCurve1, &aCurve2).unwrap();
    assert!(!anExtrema.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema.nb_extrema(), 1);
    assert_near(anExtrema.distance(1).unwrap(), 20.0_f64.sqrt(), 2.0e-8);
}

// Migrated from tests/lowalgos/extcc/bug29712_10.  Keep both argument orders
// covered because the original regression checked both extrema orientations.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_10_BoundedParallelLines() {
    let aLine1 = line(Point3::new(1.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let aLine2 = line(Point3::new(5.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let aTrimmed1 = trimmed(&aLine1, -1.0e100, 5.0);
    let aTrimmed2 = trimmed(&aLine2, 5.0, 20.0);

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aTrimmed1, &aTrimmed2).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema12.nb_extrema(), 1);
    assert_near(anExtrema12.distance(1).unwrap(), 4.0, 2.0e-8);

    let anExtrema21 = ExtremaCurveCurve::with_curves(&aTrimmed2, &aTrimmed1).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema21.nb_extrema(), 1);
    assert_near(anExtrema21.distance(1).unwrap(), 4.0, 2.0e-8);
}

// Migrated from tests/lowalgos/extcc/bug29712_20.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_20_ConcentricTrimmedCircles() {
    checkOCC29712ConcentricCircles(3.0, -3.3, -1.3);
}

// Migrated from tests/lowalgos/extcc/bug29712_30.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_30_ConcentricTrimmedCircles() {
    checkOCC29712ConcentricCircles(3.0, 4.3, 6.3);
}

// Migrated from tests/lowalgos/extcc/bug29712_40.  A short segment on the
// axis of a circle is still reported as an infinite-extrema configuration.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_40_CircleAndAxisSegment() {
    let aCircle =
        circle(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0), 5.0).cast::<dyn GeomCurve>();
    let aLine = line(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let aSegment = trimmed(&aLine, -1.0, -5.0e-8);

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aCircle, &aSegment).unwrap();
    assert!(anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_near(anExtrema12.lower_distance().unwrap(), 5.0, 1.0e-7);

    let anExtrema21 = ExtremaCurveCurve::with_curves(&aSegment, &aCircle).unwrap();
    assert!(anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_near(anExtrema21.lower_distance().unwrap(), 5.0, 1.0e-7);
}

// Migrated from tests/lowalgos/extcc/bug29712_43.  The reversed tiny line
// segment has no extrema in either argument order.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_43_CircleAndDegenerateSegment() {
    let aCircle =
        circle(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0), 5.0).cast::<dyn GeomCurve>();
    let aLine = line(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let aSegment = trimmed(&aLine, -1.5e-7, -1.0);

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aCircle, &aSegment).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema12.nb_extrema(), 0);

    let anExtrema21 = ExtremaCurveCurve::with_curves(&aSegment, &aCircle).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema21.nb_extrema(), 0);
}
