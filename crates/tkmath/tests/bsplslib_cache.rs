//! Direct translation of BSplSLib_Cache_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, clippy::approx_constant)] // Upstream names and its sqrt(2)/2 literal.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkmath::bsplslib::{self, Cache, SurfaceD2, SurfaceSpan};
use rustycad_tkmath::gp::{Point3, Vector3};

const THE_TOLERANCE: f64 = 1e-10;

//=================================================================================================
// Test fixture for BSplSLib_Cache tests
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

// NCollection_Array2<gp_Pnt> aPoles(1, rows, 1, cols) filled as aPoles(i, j) = pole(i, j).
fn pole_grid(rows: i32, cols: i32, pole: impl Fn(i32, i32) -> [f64; 3]) -> Array2<'static, Point3> {
    let mut result: Array2<'static, Point3> = Array2::with_bounds(1, rows, 1, cols).unwrap();
    for i in 1..=rows {
        for j in 1..=cols {
            let [x, y, z] = pole(i, j);
            result[(i, j)] = Point3::new(x, y, z);
        }
    }
    result
}

/// Knots {0, 1}, multiplicities and flat knots of one direction of a Bezier patch.
struct Direction {
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    flat: Array1<'static, f64>,
    degree: i32,
}

fn bezier_direction(aDegree: i32) -> Direction {
    let knots = reals(&[0.0, 1.0]);
    let mults = ints(&[aDegree + 1, aDegree + 1]);
    let mut flat: Array1<'static, f64> = Array1::with_bounds(1, 2 * (aDegree + 1)).unwrap();
    createFlatKnots(&knots, &mults, &mut flat);
    Direction {
        knots,
        mults,
        flat,
        degree: aDegree,
    }
}

// The BSplSLib direct-evaluation arguments: indices 0, non-periodic.
fn span<'r>(
    u: f64,
    v: f64,
    du: &'r Direction,
    dv: &'r Direction,
    rational: bool,
) -> SurfaceSpan<'r, 'static> {
    SurfaceSpan {
        u,
        v,
        u_index: 0,
        v_index: 0,
        u_knots: &du.knots,
        v_knots: &dv.knots,
        u_mults: Some(&du.mults),
        v_mults: Some(&dv.mults),
        u_degree: du.degree,
        v_degree: dv.degree,
        u_rat: rational,
        v_rat: rational,
        u_per: false,
        v_per: false,
    }
}

// Creates the cache and builds it at (0.5, 0.5).
fn new_cache(
    du: &Direction,
    dv: &Direction,
    aPoles: &Array2<'_, Point3>,
    aWeights: Option<&Array2<'_, f64>>,
) -> Cache {
    let mut aCache = Cache::new(
        du.degree, false, &du.flat, dv.degree, false, &dv.flat, aWeights,
    )
    .unwrap();
    aCache
        .build_cache(0.5, 0.5, &du.flat, &dv.flat, aPoles, aWeights)
        .unwrap();
    aCache
}

// Rebuilds the cache when (u, v) lies outside the cached patch.
fn locate(
    aCache: &mut Cache,
    (u, v): (f64, f64),
    (du, dv): (&Direction, &Direction),
    aPoles: &Array2<'_, Point3>,
    aWeights: Option<&Array2<'_, f64>>,
) {
    if !aCache.is_cache_valid(u, v) {
        aCache
            .build_cache(u, v, &du.flat, &dv.flat, aPoles, aWeights)
            .unwrap();
    }
}

// The parameters of `for (double u = 0.0; u <= 1.0; u += step)`, accumulated as in C++.
fn steps(step: f64) -> Vec<f64> {
    let mut result: Vec<f64> = Vec::new();
    let mut u: f64 = 0.0;
    while u <= 1.0 {
        result.push(u);
        u += step;
    }
    result
}

fn EXPECT_NEAR(a: f64, b: f64, tolerance: f64, message: &str, u: f64, v: f64) {
    assert!((a - b).abs() <= tolerance, "{message}{u}, v={v}");
}

// X, Y and Z comparisons with the "<label> <axis> mismatch at u=" messages.
fn expect_xyz(cache: [f64; 3], direct: [f64; 3], label: &str, u: f64, v: f64) {
    for (axis, name) in ["X", "Y", "Z"].iter().enumerate() {
        EXPECT_NEAR(
            cache[axis],
            direct[axis],
            THE_TOLERANCE,
            &format!("{label} {name} mismatch at u="),
            u,
            v,
        );
    }
}

fn pnt(p: Point3) -> [f64; 3] {
    [p.x(), p.y(), p.z()]
}

fn vec3(p: Vector3) -> [f64; 3] {
    [p.x(), p.y(), p.z()]
}

// Compares point, tangents and second derivatives of a D2 evaluation.
fn expect_d2(cache: &SurfaceD2, direct: &SurfaceD2, prefix: &str, u: f64, v: f64) {
    // Compare point
    expect_xyz(
        pnt(cache.p),
        pnt(direct.p),
        &format!("{prefix} point"),
        u,
        v,
    );
    // Compare tangent U
    expect_xyz(
        vec3(cache.vu),
        vec3(direct.vu),
        &format!("{prefix} tangentU"),
        u,
        v,
    );
    // Compare tangent V
    expect_xyz(
        vec3(cache.vv),
        vec3(direct.vv),
        &format!("{prefix} tangentV"),
        u,
        v,
    );
    // Compare second derivatives
    expect_xyz(
        vec3(cache.vuu),
        vec3(direct.vuu),
        &format!("{prefix} curvU"),
        u,
        v,
    );
    expect_xyz(
        vec3(cache.vvv),
        vec3(direct.vvv),
        &format!("{prefix} curvV"),
        u,
        v,
    );
    expect_xyz(
        vec3(cache.vuv),
        vec3(direct.vuv),
        &format!("{prefix} curvUV"),
        u,
        v,
    );
}

//=================================================================================================
// Non-rational surface tests
//=================================================================================================

// Create a biquadratic Bezier surface
fn biquadratic_poles() -> Array2<'static, Point3> {
    let rows: [[[f64; 3]; 3]; 3] = [
        // Row 1 (v=0)
        [[0., 0., 0.], [1., 0., 1.], [2., 0., 0.]],
        // Row 2 (v=0.5)
        [[0., 1., 1.], [1., 1., 2.], [2., 1., 1.]],
        // Row 3 (v=1)
        [[0., 2., 0.], [1., 2., 1.], [2., 2., 0.]],
    ];
    pole_grid(3, 3, |i, j| rows[j as usize - 1][i as usize - 1])
}

#[test]
fn BSplSLib_CacheTest__D0_NonRationalSurface() {
    let aPoles = biquadratic_poles();
    let (du, dv) = (bezier_direction(2), bezier_direction(2));

    // Create cache
    let mut aCache = new_cache(&du, &dv, &aPoles, None);

    // Test at several parameter values
    for u in steps(0.2) {
        for v in steps(0.2) {
            // Cached evaluation
            locate(&mut aCache, (u, v), (&du, &dv), &aPoles, None);
            let aCachePnt: Point3 = aCache.d0(u, v).unwrap();

            // Direct evaluation using BSplSLib::D0
            let aDirectPnt: Point3 =
                bsplslib::d0(&span(u, v, &du, &dv, false), &aPoles, None).unwrap();

            // Compare
            expect_xyz(pnt(aCachePnt), pnt(aDirectPnt), "D0", u, v);
        }
    }
}

#[test]
fn BSplSLib_CacheTest__D1_NonRationalSurface() {
    let aPoles = biquadratic_poles();
    let (du, dv) = (bezier_direction(2), bezier_direction(2));

    // Create cache
    let mut aCache = new_cache(&du, &dv, &aPoles, None);

    // Test at several parameter values
    for u in steps(0.2) {
        for v in steps(0.2) {
            // Cached evaluation
            locate(&mut aCache, (u, v), (&du, &dv), &aPoles, None);
            let (aCachePnt, aCacheTanU, aCacheTanV) = aCache.d1(u, v).unwrap();

            // Direct evaluation using BSplSLib::D1
            let (aDirectPnt, aDirectTanU, aDirectTanV) =
                bsplslib::d1(&span(u, v, &du, &dv, false), &aPoles, None).unwrap();

            // Compare point
            expect_xyz(pnt(aCachePnt), pnt(aDirectPnt), "D1 point", u, v);
            // Compare tangent U
            expect_xyz(vec3(aCacheTanU), vec3(aDirectTanU), "D1 tangentU", u, v);
            // Compare tangent V
            expect_xyz(vec3(aCacheTanV), vec3(aDirectTanV), "D1 tangentV", u, v);
        }
    }
}

#[test]
fn BSplSLib_CacheTest__D2_NonRationalSurface() {
    let aPoles = biquadratic_poles();
    let (du, dv) = (bezier_direction(2), bezier_direction(2));

    // Create cache
    let mut aCache = new_cache(&du, &dv, &aPoles, None);

    // Test at several parameter values
    for u in steps(0.2) {
        for v in steps(0.2) {
            // Cached evaluation
            locate(&mut aCache, (u, v), (&du, &dv), &aPoles, None);
            let aCache2: SurfaceD2 = aCache.d2(u, v).unwrap();

            // Direct evaluation using BSplSLib::D2
            let aDirect: SurfaceD2 =
                bsplslib::d2(&span(u, v, &du, &dv, false), &aPoles, None).unwrap();

            expect_d2(&aCache2, &aDirect, "D2", u, v);
        }
    }
}

include!("bsplslib_cache/suites_0.rs");
