// The comparison loops that the suite repeats test after test, over `theCounts` =
// (NbU, NbV) cells of the grid: points against a reference `Value`, and derivative
// records against reference `D1`/`D2`/`D3` records.
fn CheckPoints(
    theGrid: &Array2<'_, Point3>,
    theCounts: (i32, i32),
    theExpected: impl Fn(i32, i32) -> Point3,
) {
    for iU in 1..=theCounts.0 {
        for iV in 1..=theCounts.1 {
            EXPECT_NEAR(
                theGrid[(iU, iV)].distance(&theExpected(iU, iV)),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}

fn CheckD1(
    theGrid: &Array2<'_, GeomGridEvalSurfD1>,
    theCounts: (i32, i32),
    theExpected: impl Fn(i32, i32) -> GeomGridEvalSurfD1,
) {
    for iU in 1..=theCounts.0 {
        for iV in 1..=theCounts.1 {
            let (g, r) = (&theGrid[(iU, iV)], theExpected(iU, iV));
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

fn CheckD2(
    theGrid: &Array2<'_, GeomGridEvalSurfD2>,
    theCounts: (i32, i32),
    theExpected: impl Fn(i32, i32) -> GeomGridEvalSurfD2,
) {
    for iU in 1..=theCounts.0 {
        for iV in 1..=theCounts.1 {
            let (g, r) = (&theGrid[(iU, iV)], theExpected(iU, iV));
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2u - r.d2u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2v - r.d2v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2uv - r.d2uv).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

// The D3 checks: every member when `theAll`, otherwise the point and the four third
// derivatives only (the two shapes of the upstream loops).
fn CheckD3(
    theGrid: &Array2<'_, GeomGridEvalSurfD3>,
    theCounts: (i32, i32),
    theAll: bool,
    theExpected: impl Fn(i32, i32) -> GeomGridEvalSurfD3,
) {
    for iU in 1..=theCounts.0 {
        for iV in 1..=theCounts.1 {
            let (g, r) = (&theGrid[(iU, iV)], theExpected(iU, iV));
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            if theAll {
                EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
                EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
                EXPECT_NEAR((g.d2u - r.d2u).magnitude(), 0.0, THE_TOLERANCE);
                EXPECT_NEAR((g.d2v - r.d2v).magnitude(), 0.0, THE_TOLERANCE);
                EXPECT_NEAR((g.d2uv - r.d2uv).magnitude(), 0.0, THE_TOLERANCE);
            }
            EXPECT_NEAR((g.d3u - r.d3u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3v - r.d3v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3uuv - r.d3uuv).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3uvv - r.d3uvv).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

//=================================================================================================
// Tests for GeomGridEval_Plane
//=================================================================================================

#[test]
fn GeomGridEval_PlaneTest__BasicEvaluation() {
    // XY plane at origin
    let aPlane = PlaneZ(0.0, 0.0, 0.0);

    let anEval = GeomGridEvalPlane::new(aPlane);
    assert!(!anEval.geometry().is_null());

    let aUParams = CreateUniformParams(0.0, 5.0, 6);
    let aVParams = CreateUniformParams(0.0, 3.0, 4);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams);
    // RowLength() is the number of columns, ColLength() the number of rows.
    assert_eq!(aGrid.nb_columns(), 4);
    assert_eq!(aGrid.nb_rows(), 6);

    // Verify that all points are in Z=0 plane
    for iU in 1..=6 {
        for iV in 1..=4 {
            let aPnt: &Point3 = &aGrid[(iU, iV)];
            EXPECT_NEAR(aPnt.z(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(aPnt.x(), aUParams[iU], THE_TOLERANCE);
            EXPECT_NEAR(aPnt.y(), aVParams[iV], THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_PlaneTest__NonOriginPlane() {
    // Plane at (1, 2, 3) with normal (0, 0, 1)
    let aPlane = PlaneZ(1.0, 2.0, 3.0);
    let anEval = GeomGridEvalPlane::new(aPlane);

    let aUParams = CreateUniformParams(-1.0, 1.0, 3);
    let aVParams = CreateUniformParams(-1.0, 1.0, 3);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams);

    // All points should be at Z=3
    for iU in 1..=3 {
        for iV in 1..=3 {
            EXPECT_NEAR(aGrid[(iU, iV)].z(), 3.0, THE_TOLERANCE);
        }
    }

    // Center point (u=0, v=0) should be at (1, 2, 3)
    EXPECT_NEAR(aGrid[(2, 2)].x(), 1.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[(2, 2)].y(), 2.0, THE_TOLERANCE);
}

//=================================================================================================
// Tests for GeomGridEval_Sphere
//=================================================================================================

#[test]
fn GeomGridEval_SphereTest__BasicEvaluation() {
    // Unit sphere at origin
    let aSphere = SphereZ(0.0, 0.0, 0.0, 1.0);

    let anEval = GeomGridEvalSphere::new(aSphere);
    assert!(!anEval.geometry().is_null());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9); // Longitude
    let aVParams = CreateUniformParams(-PI / 2.0, PI / 2.0, 5); // Latitude

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // All points should be at distance 1 from origin
    for iU in 1..=9 {
        for iV in 1..=5 {
            let aDist: f64 = aGrid[(iU, iV)].distance(&Pnt(0.0, 0.0, 0.0));
            EXPECT_NEAR(aDist, 1.0, THE_TOLERANCE);
        }
    }

    // North pole (v = PI/2) should be at (0, 0, 1)
    for iU in 1..=9 {
        EXPECT_NEAR(aGrid[(iU, 5)].x(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR(aGrid[(iU, 5)].y(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR(aGrid[(iU, 5)].z(), 1.0, THE_TOLERANCE);
    }

    // South pole (v = -PI/2) should be at (0, 0, -1)
    for iU in 1..=9 {
        EXPECT_NEAR(aGrid[(iU, 1)].x(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR(aGrid[(iU, 1)].y(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR(aGrid[(iU, 1)].z(), -1.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_SphereTest__NonUnitSphere() {
    // Sphere with radius 3 at center (1, 2, 3)
    let aSphere = SphereZ(1.0, 2.0, 3.0, 3.0);
    let anEval = GeomGridEvalSphere::new(aSphere);

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 17);
    let aVParams = CreateUniformParams(-PI / 2.0, PI / 2.0, 9);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // All points should be at distance 3 from center
    let aCenter = Pnt(1.0, 2.0, 3.0);
    for iU in 1..=17 {
        for iV in 1..=9 {
            let aDist: f64 = aGrid[(iU, iV)].distance(&aCenter);
            EXPECT_NEAR(aDist, 3.0, THE_TOLERANCE);
        }
    }
}

//=================================================================================================
// Tests for GeomGridEval_OtherSurface (fallback)
//=================================================================================================

#[test]
fn GeomGridEval_OtherSurfaceTest__CylinderFallback() {
    // Cylinder is not directly supported, should use fallback
    let aCyl = CylinderOfRadius2();
    let anAdaptor = Adaptor(&aCyl);

    let anEval = GeomGridEvalOtherSurface::from_adaptor(&anAdaptor);

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, (9, 6), |iU, iV| {
        aCyl.value(aUParams[iU], aVParams[iV]).unwrap()
    });
}

//=================================================================================================
// Tests for GeomGridEval_Surface (unified dispatcher)
//=================================================================================================

#[test]
fn GeomGridEval_SurfaceTest__PlaneDispatch() {
    let aGeomPlane = PlaneZ(0.0, 0.0, 0.0);
    let anAdaptor = Adaptor(&aGeomPlane);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::Plane);

    let aUParams = CreateUniformParams(-5.0, 5.0, 11);
    let aVParams = CreateUniformParams(-3.0, 3.0, 7);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, (11, 7), |iU, iV| {
        aGeomPlane.value(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__SphereDispatch() {
    let aGeomSphere = SphereZ(0.0, 0.0, 0.0, 2.0);
    let anAdaptor = Adaptor(&aGeomSphere);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::Sphere);

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 13);
    let aVParams = CreateUniformParams(-PI / 2.0, PI / 2.0, 7);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, (13, 7), |iU, iV| {
        aGeomSphere.value(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__BSplineDispatch() {
    let aSurf = CreateSimpleBSplineSurface();
    let anAdaptor = Adaptor(&aSurf);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::BSplineSurface);

    let aUParams = CreateUniformParams(0.0, 1.0, 11);
    let aVParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, (11, 11), |iU, iV| {
        aSurf.value(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__BezierSurfaceDispatch() {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aPoles.set_value(1, 1, Pnt(0.0, 0.0, 0.0)).unwrap();
    aPoles.set_value(2, 1, Pnt(1.0, 0.0, 0.0)).unwrap();
    aPoles.set_value(1, 2, Pnt(0.0, 1.0, 0.0)).unwrap();
    aPoles.set_value(2, 2, Pnt(1.0, 1.0, 0.0)).unwrap();
    let aBezier = Handle::new(GeomBezierSurface::new(&aPoles).unwrap());
    let anAdaptor = Adaptor(&aBezier);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::BezierSurface);

    let aParams = CreateUniformParams(0.0, 1.0, 5);
    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aParams, &aParams).unwrap();

    CheckPoints(&aGrid, (5, 5), |i, j| {
        aBezier.value(aParams[i], aParams[j]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__CylinderDispatch() {
    let aCyl = CylinderOfRadius2();
    let anAdaptor = Adaptor(&aCyl);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::Cylinder);

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, (9, 6), |iU, iV| {
        aCyl.value(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__TorusDispatch() {
    let aTorus = Torus4x1();
    let anAdaptor = Adaptor(&aTorus);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::Torus);

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, (9, 9), |iU, iV| {
        aTorus.value(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__ConeDispatch() {
    let aCone = ConeQuarterPi();
    let anAdaptor = Adaptor(&aCone);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::Cone);

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, (9, 6), |iU, iV| {
        aCone.value(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__SurfaceOfRevolutionFallbackDispatch() {
    // Surface of Revolution is not optimized, should use fallback
    let aLine: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(Pnt(1.0, 0.0, 0.0), Dir(0.0, 0.0, 1.0))).cast(); // Line at x=1 parallel to Z
    let aRevSurf = Handle::new(GeomSurfaceOfRevolution::new(&aLine, gp::oz()).unwrap()); // Revolving around Z -> Cylinder-like
    let anAdaptor = Adaptor(&aRevSurf);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::SurfaceOfRevolution);

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, (9, 6), |iU, iV| {
        aRevSurf.value(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__DirectHandleInit() {
    let aPlane = PlaneZ(0.0, 0.0, 0.0);

    // Initialize directly from occ::handle<Geom_Surface>
    let anEval = GeomGridEvalSurface::from_surface(&aPlane.cast());
    assert_eq!(anEval.get_type(), SurfaceType::Plane);

    let aUParams = CreateUniformParams(0.0, 1.0, 5);
    let aVParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();
    assert!(!aGrid.is_empty());

    // Verify value
    EXPECT_NEAR(aGrid[(1, 1)].z(), 0.0, THE_TOLERANCE);
}

#[test]
fn GeomGridEval_SurfaceTest__EmptyParams() {
    let aGeomPlane = PlaneZ(0.0, 0.0, 0.0);
    let anAdaptor = Adaptor(&aGeomPlane);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();

    // EvaluateGrid with empty params should return empty
    let aEmptyParams: Array1<'static, f64> = Array1::default();
    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aEmptyParams, &aEmptyParams).unwrap();
    assert!(aGrid.is_empty());
}
