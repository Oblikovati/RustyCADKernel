//==================================================================================================
// Bezier with different configurations
//==================================================================================================

/// A Bezier curve from its poles.
fn bezier(poles: &[Point3]) -> Handle<GeomBezierCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, poles.len() as i32).unwrap();
    for (i, p) in poles.iter().enumerate() {
        aPoles[i as i32 + 1] = *p;
    }
    Handle::new(GeomBezierCurve::new(&aPoles).unwrap())
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Bezier_Quadratic() {
    let aBezier = bezier(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(5.0, 10.0, 0.0),
        Point3::new(10.0, 0.0, 0.0),
    ]);
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBezier.cast()).unwrap();

    let aPoint = Point3::new(5.0, 8.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Bezier_Quadratic", 0.01)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Bezier_HighDegree() {
    // Degree 6 Bezier
    let aBezier = bezier(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 3.0, 0.0),
        Point3::new(2.0, -1.0, 0.0),
        Point3::new(3.0, 4.0, 0.0),
        Point3::new(4.0, 0.0, 0.0),
        Point3::new(5.0, 2.0, 0.0),
        Point3::new(6.0, 1.0, 0.0),
    ]);
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBezier.cast()).unwrap();

    let aPoint = Point3::new(3.0, 3.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Bezier_HighDegree", 0.01)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Bezier_3D_SCurve() {
    // 3D S-curve
    let aBezier = bezier(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(3.0, 5.0, 2.0),
        Point3::new(6.0, 0.0, 4.0),
        Point3::new(9.0, 5.0, 6.0),
    ]);
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBezier.cast()).unwrap();

    let aPoint = Point3::new(4.5, 3.0, 3.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Bezier_3D_SCurve", 0.01)
    });
}

//==================================================================================================
// Offset curves with different bases
//==================================================================================================

#[test]
fn ExtremaPC_ExtendedGeometryTest__OffsetEllipse() {
    let aBaseEllipse = Handle::new(
        GeomEllipse::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 20.0, 10.0)
            .unwrap(),
    );
    let anAdaptor = whole(
        GeomOffsetCurve::new(&aBaseEllipse.cast(), 3.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );

    let aPoint = Point3::new(30.0, 5.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "OffsetEllipse", 0.05)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__OffsetBezier() {
    let aBaseBezier = bezier(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 4.0, 0.0),
        Point3::new(4.0, 4.0, 0.0),
        Point3::new(6.0, 0.0, 0.0),
    ]);
    let anAdaptor = whole(
        GeomOffsetCurve::new(&aBaseBezier.cast(), 1.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );

    let aPoint = Point3::new(3.0, 5.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "OffsetBezier", 0.05)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__OffsetNegative() {
    // Offset with negative distance (inside)
    let aBaseCircle = Handle::new(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 15.0).unwrap(),
    );
    let anAdaptor = whole(
        GeomOffsetCurve::new(&aBaseCircle.cast(), -5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );

    let aPoint = Point3::new(20.0, 0.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "OffsetNegative", 0.01);
        EXPECT_NEAR(aNewResult.min_square_distance().sqrt(), 10.0, 0.01); // 20 - 10 = 10
    });
}

//==================================================================================================
// Arc (partial curve) tests
//==================================================================================================

#[test]
fn ExtremaPC_ExtendedGeometryTest__CircleArc_FirstQuadrant() {
    let anAdaptor = ranged(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
        0.0,
        PI / 2.0,
    );

    let aPoint = Point3::new(15.0, 15.0, 0.0);

    // Only compare minimum - max may differ between implementations for arcs
    // due to endpoint handling
    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "CircleArc_FirstQuadrant", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__CircleArc_ThirdQuadrant() {
    let anAdaptor = ranged(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
        PI,
        3.0 * PI / 2.0,
    );

    let aPoint = Point3::new(-15.0, -5.0, 0.0);

    // Only compare minimum - max may differ between implementations for arcs
    // due to endpoint handling
    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "CircleArc_ThirdQuadrant", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__EllipseArc_SemiEllipse() {
    let anAdaptor = ranged(
        GeomEllipse::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 20.0, 10.0)
            .unwrap(),
        0.0,
        PI,
    );

    let aPoint = Point3::new(0.0, 20.0, 0.0);

    // Only compare minimum - max may differ between implementations for arcs
    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "EllipseArc_SemiEllipse", THE_TOL)
    });
}

//==================================================================================================
// Edge cases and degenerate situations
//==================================================================================================

#[test]
fn ExtremaPC_ExtendedGeometryTest__Point_ExactlyOnCurve_Circle() {
    let anAdaptor = whole(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );

    let aPoint = Point3::new(10.0, 0.0, 0.0); // Exactly on circle

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aNewResult.is_done());
    EXPECT_NEAR(aNewResult.min_square_distance().sqrt(), 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Point_ExactlyOnCurve_BSpline() {
    let aPoles = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 2.0, 0.0),
        Point3::new(3.0, 2.0, 0.0),
        Point3::new(4.0, 0.0, 0.0),
    ];
    let anAdaptor = bspline(&aPoles, &[0.0, 1.0], &[4, 4], 3);

    // Point exactly on curve at t=0.5
    let aPoint: Point3 = anAdaptor.value(0.5).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aNewResult.is_done());
    EXPECT_NEAR(aNewResult.min_square_distance().sqrt(), 0.0, 0.01);
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Point_VeryFar() {
    let anAdaptor = whole(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );

    let aPoint = Point3::new(10000.0, 10000.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Point_VeryFar", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Point_VeryClose() {
    let anAdaptor = whole(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );

    let aPoint = Point3::new(10.0001, 0.0, 0.0); // Very close to curve

    compare_point(&anAdaptor, aPoint, |aNewResult, _anOldExtPC| {
        assert!(aNewResult.is_done());
        EXPECT_NEAR(aNewResult.min_square_distance().sqrt(), 0.0001, 1.0e-5);
    });
}

//==================================================================================================
// Random stress tests for all geometry types
//==================================================================================================

// The pass count of the circle and ellipse random tests: points whose minima agree
// within THE_TOL.
fn random_pass_count(anAdaptor: &GeomAdaptorCurve, aPoints: &[Point3]) -> i32 {
    let mut aPassCount: i32 = 0;
    for &aPoint in aPoints {
        let mut anExtPC = ExtremaPC::Curve::from_adaptor(anAdaptor).unwrap();
        let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

        let anOldExtPC = old_ext_pc(aPoint, anAdaptor);

        if !anOldExtPC.is_done() {
            continue;
        }

        if aNewResult.is_done() && aNewResult.nb_ext() > 0 {
            let aNewMin: f64 = aNewResult.min_square_distance().sqrt();
            let mut aOldMinSq: f64 = f64::MAX;
            for j in 1..=anOldExtPC.nb_ext().unwrap() {
                if anOldExtPC.square_distance(j).unwrap() < aOldMinSq {
                    aOldMinSq = anOldExtPC.square_distance(j).unwrap();
                }
            }
            let aOldMin: f64 = aOldMinSq.sqrt();
            if (aNewMin - aOldMin).abs() < THE_TOL {
                aPassCount += 1;
            }
        }
    }
    aPassCount
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Random_Circle_100Points() {
    let anAdaptor = whole(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );

    // std::mt19937 aGen(12345), uniform_real_distribution<double>(-100.0, 100.0)
    let aPoints = random_points("extended_circle");
    assert_eq!(aPoints.len(), 100);

    let aPassCount: i32 = random_pass_count(&anAdaptor, &aPoints);

    EXPECT_GE(aPassCount, 95); // At least 95% of random tests should pass
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Random_Ellipse_100Points() {
    let anAdaptor = whole(
        GeomEllipse::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 20.0, 10.0)
            .unwrap(),
    );

    // std::mt19937 aGen(54321), uniform_real_distribution<double>(-100.0, 100.0)
    let aPoints = random_points("extended_ellipse");
    assert_eq!(aPoints.len(), 100);

    let aPassCount: i32 = random_pass_count(&anAdaptor, &aPoints);

    EXPECT_GE(aPassCount, 95); // At least 95% of random tests should pass
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Random_Parabola_100Points() {
    let anAdaptor = ranged(
        GeomParabola::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 5.0).unwrap(),
        -20.0,
        20.0,
    );

    // std::mt19937 aGen(11111), uniform_real_distribution<double>(-50.0, 50.0)
    let aPoints = random_points("extended_parabola");
    assert_eq!(aPoints.len(), 100);

    let mut aPassCount: i32 = 0;
    let mut aValidCount: i32 = 0;
    for &aPoint in &aPoints {
        let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
        let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

        let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

        if !anOldExtPC.is_done() || anOldExtPC.nb_ext().unwrap() == 0 {
            continue;
        }

        if !aNewResult.is_done() || aNewResult.nb_ext() == 0 {
            continue;
        }

        aValidCount += 1;

        let aNewMin: f64 = aNewResult.min_square_distance().sqrt();
        let mut aOldMin: f64 = f64::MAX;
        for j in 1..=anOldExtPC.nb_ext().unwrap() {
            if anOldExtPC.square_distance(j).unwrap() < aOldMin * aOldMin {
                aOldMin = anOldExtPC.square_distance(j).unwrap().sqrt();
            }
        }
        // Use relaxed tolerance for random tests
        if (aNewMin - aOldMin).abs() < 0.01 {
            aPassCount += 1;
        }
    }

    // At least 80% of valid random tests should pass
    if aValidCount > 0 {
        let aPassRate: f64 = f64::from(aPassCount) / f64::from(aValidCount);
        EXPECT_GE(aPassRate, 0.80); // At least 80% of random tests should pass
    }
}
