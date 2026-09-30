//! Direct translation of BSplCLib_Cache_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::bsplclib::{self, Cache, CurveSpan};
use rustycad_tkmath::gp::{Point3, Vector3};

const THE_TOLERANCE: f64 = 1e-10;

//=================================================================================================
// Test fixture for BSplCLib_Cache tests
//=================================================================================================

/// Creates flat knots array from knots and multiplicities
fn createFlatKnots(
    theKnots: &Array1<'_, f64>,
    theMults: &Array1<'_, i32>,
    theFlatKnots: &mut Array1<'_, f64>,
) {
    let mut aFlatIndex: i32 = theFlatKnots.lower();
    for i in theKnots.lower()..=theKnots.upper() {
        for _j in 0..theMults[i] {
            theFlatKnots[aFlatIndex] = theKnots[i];
            aFlatIndex += 1;
        }
    }
}

fn points(values: &[[f64; 3]]) -> Array1<'static, Point3> {
    let mut result: Array1<'static, Point3> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, [x, y, z]) in values.iter().enumerate() {
        result[i as i32 + 1] = Point3::new(*x, *y, *z);
    }
    result
}

fn reals(values: &[f64]) -> Array1<'static, f64> {
    let mut result: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

fn ints(values: &[i32]) -> Array1<'static, i32> {
    let mut result: Array1<'static, i32> = Array1::with_bounds(1, values.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

fn span<'r, 'a>(
    u: f64,
    degree: i32,
    knots: &'r Array1<'a, f64>,
    mults: &'r Array1<'a, i32>,
) -> CurveSpan<'r, 'a> {
    CurveSpan {
        u,
        index: 0,
        degree,
        periodic: false,
        knots,
        mults: Some(mults),
    }
}

fn EXPECT_NEAR(a: f64, b: f64, tolerance: f64, message: &str, u: f64) {
    assert!((a - b).abs() <= tolerance, "{message}{u}");
}

//=================================================================================================
// Non-rational 3D curve tests
//=================================================================================================

#[test]
fn BSplCLib_CacheTest__D0_NonRationalCurve3D() {
    // Create a cubic Bezier curve
    let aPoles = points(&[[0., 0., 0.], [1., 2., 0.], [2., 2., 0.], [3., 0., 0.]]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = ints(&[4, 4]);
    let mut aFlatKnots: Array1<'static, f64> = Array1::with_bounds(1, 8).unwrap();
    createFlatKnots(&aKnots, &aMults, &mut aFlatKnots);

    let aDegree: i32 = 3;

    // Create cache
    let mut aCache = Cache::new(aDegree, false, &aFlatKnots, &aPoles, None).unwrap();
    aCache.build_cache(0.5, &aFlatKnots, &aPoles, None).unwrap();

    // Test at several parameter values
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        // Cached evaluation
        if !aCache.is_cache_valid(u) {
            aCache.build_cache(u, &aFlatKnots, &aPoles, None).unwrap();
        }
        let aCachePnt: Point3 = aCache.d0(u).unwrap();

        // Direct evaluation using BSplCLib::D0
        let aDirectPnt: Point3 =
            bsplclib::d0(&span(u, aDegree, &aKnots, &aMults), &aPoles, None).unwrap();

        // Compare
        EXPECT_NEAR(
            aCachePnt.x(),
            aDirectPnt.x(),
            THE_TOLERANCE,
            "D0 X mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCachePnt.y(),
            aDirectPnt.y(),
            THE_TOLERANCE,
            "D0 Y mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCachePnt.z(),
            aDirectPnt.z(),
            THE_TOLERANCE,
            "D0 Z mismatch at u=",
            u,
        );
        u += 0.1;
    }
}

#[test]
fn BSplCLib_CacheTest__D1_NonRationalCurve3D() {
    // Create a cubic Bezier curve
    let aPoles = points(&[[0., 0., 0.], [1., 2., 1.], [2., 2., 1.], [3., 0., 0.]]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = ints(&[4, 4]);
    let mut aFlatKnots: Array1<'static, f64> = Array1::with_bounds(1, 8).unwrap();
    createFlatKnots(&aKnots, &aMults, &mut aFlatKnots);

    let aDegree: i32 = 3;

    // Create cache
    let mut aCache = Cache::new(aDegree, false, &aFlatKnots, &aPoles, None).unwrap();
    aCache.build_cache(0.5, &aFlatKnots, &aPoles, None).unwrap();

    // Test at several parameter values
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        // Cached evaluation
        if !aCache.is_cache_valid(u) {
            aCache.build_cache(u, &aFlatKnots, &aPoles, None).unwrap();
        }
        let (aCachePnt, aCacheTan): (Point3, Vector3) = aCache.d1(u).unwrap();

        // Direct evaluation using BSplCLib::D1
        let (aDirectPnt, aDirectTan): (Point3, Vector3) =
            bsplclib::d1(&span(u, aDegree, &aKnots, &aMults), &aPoles, None).unwrap();

        // Compare point
        EXPECT_NEAR(
            aCachePnt.x(),
            aDirectPnt.x(),
            THE_TOLERANCE,
            "D1 point X mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCachePnt.y(),
            aDirectPnt.y(),
            THE_TOLERANCE,
            "D1 point Y mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCachePnt.z(),
            aDirectPnt.z(),
            THE_TOLERANCE,
            "D1 point Z mismatch at u=",
            u,
        );

        // Compare tangent
        EXPECT_NEAR(
            aCacheTan.x(),
            aDirectTan.x(),
            THE_TOLERANCE,
            "D1 tangent X mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCacheTan.y(),
            aDirectTan.y(),
            THE_TOLERANCE,
            "D1 tangent Y mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCacheTan.z(),
            aDirectTan.z(),
            THE_TOLERANCE,
            "D1 tangent Z mismatch at u=",
            u,
        );
        u += 0.1;
    }
}

#[test]
fn BSplCLib_CacheTest__D2_NonRationalCurve3D() {
    // Create a cubic Bezier curve
    let aPoles = points(&[[0., 0., 0.], [1., 2., 1.], [2., 2., 1.], [3., 0., 0.]]);
    let aKnots = reals(&[0.0, 1.0]);
    let aMults = ints(&[4, 4]);
    let mut aFlatKnots: Array1<'static, f64> = Array1::with_bounds(1, 8).unwrap();
    createFlatKnots(&aKnots, &aMults, &mut aFlatKnots);

    let aDegree: i32 = 3;

    // Create cache
    let mut aCache = Cache::new(aDegree, false, &aFlatKnots, &aPoles, None).unwrap();
    aCache.build_cache(0.5, &aFlatKnots, &aPoles, None).unwrap();

    // Test at several parameter values
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        // Cached evaluation
        if !aCache.is_cache_valid(u) {
            aCache.build_cache(u, &aFlatKnots, &aPoles, None).unwrap();
        }
        let (aCachePnt, aCacheTan, aCacheCurv): (Point3, Vector3, Vector3) = aCache.d2(u).unwrap();

        // Direct evaluation using BSplCLib::D2
        let (aDirectPnt, aDirectTan, aDirectCurv): (Point3, Vector3, Vector3) =
            bsplclib::d2(&span(u, aDegree, &aKnots, &aMults), &aPoles, None).unwrap();

        // Compare point
        EXPECT_NEAR(
            aCachePnt.x(),
            aDirectPnt.x(),
            THE_TOLERANCE,
            "D2 point X mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCachePnt.y(),
            aDirectPnt.y(),
            THE_TOLERANCE,
            "D2 point Y mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCachePnt.z(),
            aDirectPnt.z(),
            THE_TOLERANCE,
            "D2 point Z mismatch at u=",
            u,
        );

        // Compare tangent
        EXPECT_NEAR(
            aCacheTan.x(),
            aDirectTan.x(),
            THE_TOLERANCE,
            "D2 tangent X mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCacheTan.y(),
            aDirectTan.y(),
            THE_TOLERANCE,
            "D2 tangent Y mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCacheTan.z(),
            aDirectTan.z(),
            THE_TOLERANCE,
            "D2 tangent Z mismatch at u=",
            u,
        );

        // Compare curvature
        EXPECT_NEAR(
            aCacheCurv.x(),
            aDirectCurv.x(),
            THE_TOLERANCE,
            "D2 curvature X mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCacheCurv.y(),
            aDirectCurv.y(),
            THE_TOLERANCE,
            "D2 curvature Y mismatch at u=",
            u,
        );
        EXPECT_NEAR(
            aCacheCurv.z(),
            aDirectCurv.z(),
            THE_TOLERANCE,
            "D2 curvature Z mismatch at u=",
            u,
        );
        u += 0.1;
    }
}

include!("bsplclib_cache/suites_0.rs");
