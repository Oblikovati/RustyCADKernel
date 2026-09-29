//==================================================================================================
// Parabola tests with different configurations
//==================================================================================================

#[test]
fn ExtremaPC_ExtendedGeometryTest__Parabola_Translated() {
    let anAdaptor = ranged(
        GeomParabola::new(ax2(Point3::new(10.0, 20.0, 0.0), dir3(0.0, 0.0, 1.0)), 3.0).unwrap(),
        -15.0,
        15.0,
    );

    let aPoint = Point3::new(20.0, 30.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Parabola_Translated", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Parabola_InXZPlane() {
    let anAdaptor = ranged(
        GeomParabola::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 1.0, 0.0)), 5.0).unwrap(),
        -10.0,
        10.0,
    );

    let aPoint = Point3::new(5.0, 5.0, 10.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Parabola_InXZPlane", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Parabola_SmallFocalLength() {
    let anAdaptor = ranged(
        GeomParabola::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 0.5).unwrap(),
        -5.0,
        5.0,
    );

    let aPoint = Point3::new(2.0, 3.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Parabola_SmallFocalLength", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Parabola_LargeFocalLength() {
    let anAdaptor = ranged(
        GeomParabola::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 50.0).unwrap(),
        -100.0,
        100.0,
    );

    let aPoint = Point3::new(50.0, 25.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Parabola_LargeFocalLength", THE_TOL)
    });
}

//==================================================================================================
// Hyperbola tests with different configurations
//==================================================================================================

#[test]
fn ExtremaPC_ExtendedGeometryTest__Hyperbola_Translated() {
    let anAdaptor = ranged(
        GeomHyperbola::new(ax2(Point3::new(50.0, 50.0, 0.0), dir3(0.0, 0.0, 1.0)), 15.0, 8.0)
            .unwrap(),
        -1.5,
        1.5,
    );

    let aPoint = Point3::new(80.0, 50.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Hyperbola_Translated", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Hyperbola_InXZPlane() {
    let anAdaptor = ranged(
        GeomHyperbola::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 1.0, 0.0)), 10.0, 5.0)
            .unwrap(),
        -2.0,
        2.0,
    );

    let aPoint = Point3::new(20.0, 5.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Hyperbola_InXZPlane", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Hyperbola_HighEccentricity() {
    // Hyperbola with high eccentricity (a >> b)
    let anAdaptor = ranged(
        GeomHyperbola::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 30.0, 3.0)
            .unwrap(),
        -1.0,
        1.0,
    );

    let aPoint = Point3::new(50.0, 10.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aNewResult.is_done());
    assert!(aNewResult.nb_ext() > 0);

    // Verify that the minimum extremum is correct by checking
    // the computed point is actually on the curve
    let aMinIdx: usize = aNewResult.min_index();
    let aMinPoint: Point3 = aNewResult[aMinIdx].point;

    // Verify the distance is consistent
    let aComputedSqDist: f64 = aPoint.square_distance(&aMinPoint);
    EXPECT_NEAR(aComputedSqDist, aNewResult[aMinIdx].square_distance, THE_TOL);
}

//==================================================================================================
// BSpline with different configurations
//==================================================================================================

/// A B-spline from its poles, knots, multiplicities and degree.
fn bspline(poles: &[Point3], knots: &[f64], mults: &[i32], degree: i32) -> GeomAdaptorCurve {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, poles.len() as i32).unwrap();
    for (i, p) in poles.iter().enumerate() {
        aPoles[i as i32 + 1] = *p;
    }
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, knots.len() as i32).unwrap();
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, mults.len() as i32).unwrap();
    for i in 0..knots.len() {
        aKnots[i as i32 + 1] = knots[i];
        aMults[i as i32 + 1] = mults[i];
    }
    whole(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, degree, false).unwrap())
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__BSpline_Linear() {
    // Degree 1 BSpline (linear segments)
    let aPoles = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(3.0, 1.0, 0.0),
    ];
    let anAdaptor = bspline(&aPoles, &[0.0, 0.33, 0.67, 1.0], &[2, 1, 1, 2], 1);

    let aPoint = Point3::new(1.5, 1.5, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "BSpline_Linear", 0.01)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__BSpline_Quadratic() {
    // Degree 2 BSpline
    let aPoles = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 2.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(3.0, 2.0, 0.0),
        Point3::new(4.0, 0.0, 0.0),
    ];
    let anAdaptor = bspline(&aPoles, &[0.0, 0.5, 1.0], &[3, 2, 3], 2);

    let aPoint = Point3::new(2.0, 2.5, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "BSpline_Quadratic", 0.01)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__BSpline_3D_Helix() {
    // 3D helix-like BSpline
    let aPoles = [
        Point3::new(10.0, 0.0, 0.0),
        Point3::new(7.0, 7.0, 5.0),
        Point3::new(0.0, 10.0, 10.0),
        Point3::new(-7.0, 7.0, 15.0),
        Point3::new(-10.0, 0.0, 20.0),
        Point3::new(-7.0, -7.0, 25.0),
        Point3::new(0.0, -10.0, 30.0),
    ];
    let anAdaptor = bspline(&aPoles, &[0.0, 1.0], &[7, 7], 6);

    let aPoint = Point3::new(0.0, 0.0, 15.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "BSpline_3D_Helix", 0.05)
    });
}
