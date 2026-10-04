//=================================================================================================
// Rational 3D curve tests
//=================================================================================================

// A rational quadratic Bezier curve (can represent a circle arc).
fn rational_arc() -> (
    Array1<'static, Point3>,
    Array1<'static, f64>,
    Array1<'static, f64>,
    Array1<'static, i32>,
) {
    let aPoles = points(&[[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
    let aWeights = reals(&[1.0, 0.707106781186548, 1.0]); // sqrt(2)/2
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = ints(&[3, 3]);
    (aPoles, aWeights, aKnots, aMults)
}

#[test]
fn BSplCLib_CacheTest__D0_RationalCurve3D() {
    // Create a rational quadratic Bezier curve (can represent a circle arc)
    let (aPoles, aWeights, aKnots, aMults) = rational_arc();
    let mut aFlatKnots: Array1<'static, f64> = Array1::with_bounds(1, 6).unwrap();
    createFlatKnots(&aKnots, &aMults, &mut aFlatKnots);

    let aDegree: i32 = 2;

    // Create cache
    let aCache = Cache::new(aDegree, false, &aFlatKnots, &aPoles, Some(&aWeights)).unwrap();
    aCache.build_cache(0.5, &aFlatKnots, &aPoles, Some(&aWeights)).unwrap();

    // Test at several parameter values
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        // Cached evaluation
        if !aCache.is_cache_valid(u) {
            aCache.build_cache(u, &aFlatKnots, &aPoles, Some(&aWeights)).unwrap();
        }
        let aCachePnt: Point3 = aCache.d0(u).unwrap();

        // Direct evaluation using BSplCLib::D0
        let aDirectPnt: Point3 =
            bsplclib::d0(&span(u, aDegree, &aKnots, &aMults), &aPoles, Some(&aWeights)).unwrap();

        // Compare
        EXPECT_NEAR(aCachePnt.x(), aDirectPnt.x(), THE_TOLERANCE, "Rational D0 X mismatch at u=", u);
        EXPECT_NEAR(aCachePnt.y(), aDirectPnt.y(), THE_TOLERANCE, "Rational D0 Y mismatch at u=", u);
        EXPECT_NEAR(aCachePnt.z(), aDirectPnt.z(), THE_TOLERANCE, "Rational D0 Z mismatch at u=", u);
        u += 0.1;
    }
}

#[test]
fn BSplCLib_CacheTest__D1_RationalCurve3D() {
    // Create a rational quadratic Bezier curve
    let (aPoles, aWeights, aKnots, aMults) = rational_arc();
    let mut aFlatKnots: Array1<'static, f64> = Array1::with_bounds(1, 6).unwrap();
    createFlatKnots(&aKnots, &aMults, &mut aFlatKnots);

    let aDegree: i32 = 2;

    // Create cache
    let aCache = Cache::new(aDegree, false, &aFlatKnots, &aPoles, Some(&aWeights)).unwrap();
    aCache.build_cache(0.5, &aFlatKnots, &aPoles, Some(&aWeights)).unwrap();

    // Test at several parameter values
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        // Cached evaluation
        if !aCache.is_cache_valid(u) {
            aCache.build_cache(u, &aFlatKnots, &aPoles, Some(&aWeights)).unwrap();
        }
        let (aCachePnt, aCacheTan): (Point3, Vector3) = aCache.d1(u).unwrap();

        // Direct evaluation using BSplCLib::D1
        let (aDirectPnt, aDirectTan): (Point3, Vector3) =
            bsplclib::d1(&span(u, aDegree, &aKnots, &aMults), &aPoles, Some(&aWeights)).unwrap();

        // Compare point
        EXPECT_NEAR(aCachePnt.x(), aDirectPnt.x(), THE_TOLERANCE, "Rational D1 point X mismatch at u=", u);
        EXPECT_NEAR(aCachePnt.y(), aDirectPnt.y(), THE_TOLERANCE, "Rational D1 point Y mismatch at u=", u);
        EXPECT_NEAR(aCachePnt.z(), aDirectPnt.z(), THE_TOLERANCE, "Rational D1 point Z mismatch at u=", u);

        // Compare tangent
        EXPECT_NEAR(aCacheTan.x(), aDirectTan.x(), THE_TOLERANCE, "Rational D1 tangent X mismatch at u=", u);
        EXPECT_NEAR(aCacheTan.y(), aDirectTan.y(), THE_TOLERANCE, "Rational D1 tangent Y mismatch at u=", u);
        EXPECT_NEAR(aCacheTan.z(), aDirectTan.z(), THE_TOLERANCE, "Rational D1 tangent Z mismatch at u=", u);
        u += 0.1;
    }
}

#[test]
fn BSplCLib_CacheTest__D2_RationalCurve3D() {
    // Create a rational quadratic Bezier curve
    let (aPoles, aWeights, aKnots, aMults) = rational_arc();
    let mut aFlatKnots: Array1<'static, f64> = Array1::with_bounds(1, 6).unwrap();
    createFlatKnots(&aKnots, &aMults, &mut aFlatKnots);

    let aDegree: i32 = 2;

    // Create cache
    let aCache = Cache::new(aDegree, false, &aFlatKnots, &aPoles, Some(&aWeights)).unwrap();
    aCache.build_cache(0.5, &aFlatKnots, &aPoles, Some(&aWeights)).unwrap();

    // Test at several parameter values
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        // Cached evaluation
        if !aCache.is_cache_valid(u) {
            aCache.build_cache(u, &aFlatKnots, &aPoles, Some(&aWeights)).unwrap();
        }
        let (aCachePnt, aCacheTan, aCacheCurv): (Point3, Vector3, Vector3) = aCache.d2(u).unwrap();

        // Direct evaluation using BSplCLib::D2
        let (aDirectPnt, aDirectTan, aDirectCurv): (Point3, Vector3, Vector3) =
            bsplclib::d2(&span(u, aDegree, &aKnots, &aMults), &aPoles, Some(&aWeights)).unwrap();

        // Compare point
        EXPECT_NEAR(aCachePnt.x(), aDirectPnt.x(), THE_TOLERANCE, "Rational D2 point X mismatch at u=", u);
        EXPECT_NEAR(aCachePnt.y(), aDirectPnt.y(), THE_TOLERANCE, "Rational D2 point Y mismatch at u=", u);
        EXPECT_NEAR(aCachePnt.z(), aDirectPnt.z(), THE_TOLERANCE, "Rational D2 point Z mismatch at u=", u);

        // Compare tangent
        EXPECT_NEAR(aCacheTan.x(), aDirectTan.x(), THE_TOLERANCE, "Rational D2 tangent X mismatch at u=", u);
        EXPECT_NEAR(aCacheTan.y(), aDirectTan.y(), THE_TOLERANCE, "Rational D2 tangent Y mismatch at u=", u);
        EXPECT_NEAR(aCacheTan.z(), aDirectTan.z(), THE_TOLERANCE, "Rational D2 tangent Z mismatch at u=", u);

        // Compare curvature
        EXPECT_NEAR(aCacheCurv.x(), aDirectCurv.x(), THE_TOLERANCE, "Rational D2 curvature X mismatch at u=", u);
        EXPECT_NEAR(aCacheCurv.y(), aDirectCurv.y(), THE_TOLERANCE, "Rational D2 curvature Y mismatch at u=", u);
        EXPECT_NEAR(aCacheCurv.z(), aDirectCurv.z(), THE_TOLERANCE, "Rational D2 curvature Z mismatch at u=", u);
        u += 0.1;
    }
}

//=================================================================================================
// Test D3 for completeness
//=================================================================================================

#[test]
fn BSplCLib_CacheTest__D3_NonRationalCurve3D() {
    // Create a cubic Bezier curve
    let aPoles = points(&[[0., 0., 0.], [1., 2., 1.], [2., 2., 1.], [3., 0., 0.]]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = ints(&[4, 4]);
    let mut aFlatKnots: Array1<'static, f64> = Array1::with_bounds(1, 8).unwrap();
    createFlatKnots(&aKnots, &aMults, &mut aFlatKnots);

    let aDegree: i32 = 3;

    // Create cache
    let aCache = Cache::new(aDegree, false, &aFlatKnots, &aPoles, None).unwrap();
    aCache.build_cache(0.5, &aFlatKnots, &aPoles, None).unwrap();

    // Test at several parameter values
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        // Cached evaluation
        if !aCache.is_cache_valid(u) {
            aCache.build_cache(u, &aFlatKnots, &aPoles, None).unwrap();
        }
        let (aCachePnt, aCacheTan, aCacheCurv, aCacheTors): (Point3, Vector3, Vector3, Vector3) =
            aCache.d3(u).unwrap();

        // Direct evaluation using BSplCLib::D3
        let (aDirectPnt, aDirectTan, aDirectCurv, aDirectTors): (Point3, Vector3, Vector3, Vector3) =
            bsplclib::d3(&span(u, aDegree, &aKnots, &aMults), &aPoles, None).unwrap();

        // Compare point
        EXPECT_NEAR(aCachePnt.x(), aDirectPnt.x(), THE_TOLERANCE, "D3 point X mismatch at u=", u);

        // Compare tangent
        EXPECT_NEAR(aCacheTan.x(), aDirectTan.x(), THE_TOLERANCE, "D3 tangent X mismatch at u=", u);

        // Compare curvature
        EXPECT_NEAR(aCacheCurv.x(), aDirectCurv.x(), THE_TOLERANCE, "D3 curvature X mismatch at u=", u);

        // Compare torsion
        EXPECT_NEAR(aCacheTors.x(), aDirectTors.x(), THE_TOLERANCE, "D3 torsion X mismatch at u=", u);
        u += 0.1;
    }
}
