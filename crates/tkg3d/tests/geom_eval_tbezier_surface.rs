//! Direct translation of GeomEval_TBezierSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{
    GeomGeometry, GeomSphericalSurface, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use rustycad_tkg3d::geom_eval::GeomEvalTBezierSurface;
use rustycad_tkmath::gp::{Axis3, Point3, Transform3, Vector3, Xyz};
use std::f64::consts::PI;

const THE_FD_TOL_D1: f64 = 1e-5;
const THE_FD_TOL_D2: f64 = 1e-4;

fn grid(rows: i32, cols: i32) -> Array2<'static, Point3> {
    Array2::with_bounds(1, rows, 1, cols).unwrap()
}

// Helper to create a T-Bezier surface representing a unit sphere patch.
// Basis: {1, sin(u), cos(u)} x {1, sin(v), cos(v)} with alpha=1.
// S(u,v) = cos(u)*cos(v)*X + sin(u)*cos(v)*Y + sin(v)*Z, domain [0, Pi]x[0, Pi].
fn createSpherePatch() -> GeomEvalTBezierSurface {
    let mut aPoles = grid(3, 3);
    // All poles default to origin (0,0,0)
    for i in 1..=3 {
        for j in 1..=3 {
            aPoles.set_value(i, j, Point3::new(0.0, 0.0, 0.0)).unwrap();
        }
    }
    // P(1,2) = (0,0,1) corresponds to 1*sin(v) -> Z component
    aPoles.set_value(1, 2, Point3::new(0.0, 0.0, 1.0)).unwrap();
    // P(2,3) = (0,1,0) corresponds to sin(u)*cos(v) -> Y component
    aPoles.set_value(2, 3, Point3::new(0.0, 1.0, 0.0)).unwrap();
    // P(3,3) = (1,0,0) corresponds to cos(u)*cos(v) -> X component
    aPoles.set_value(3, 3, Point3::new(1.0, 0.0, 0.0)).unwrap();
    GeomEvalTBezierSurface::new(&aPoles, 1.0, 1.0).unwrap()
}

// Helper to create a simple 3x3 (order 1 in both directions) T-Bezier surface
fn createSimpleSurface() -> GeomEvalTBezierSurface {
    let mut aPoles = grid(3, 3);
    for i in 1..=3 {
        for j in 1..=3 {
            let aPole = Point3::new(f64::from(i - 1), f64::from(j - 1), 0.0);
            aPoles.set_value(i, j, aPole).unwrap();
        }
    }
    GeomEvalTBezierSurface::new(&aPoles, 1.0, 1.0).unwrap()
}

// `(aPlus - aMinus) / (2 * Precision::Confusion())`.
fn central(aPlus: Xyz, aMinus: Xyz) -> Vector3 {
    Vector3::from((aPlus - aMinus) / (2.0 * precision::confusion()))
}

fn expect_vec_near(a: Vector3, b: Vector3, tolerance: f64) {
    EXPECT_NEAR(a.x(), b.x(), tolerance);
    EXPECT_NEAR(a.y(), b.y(), tolerance);
    EXPECT_NEAR(a.z(), b.z(), tolerance);
}

// Test construction with valid parameters
#[test]
fn GeomEval_TBezierSurfaceTest__Construction_ValidParams() {
    let aSurf = createSimpleSurface();
    assert_eq!(aSurf.nb_u_poles(), 3);
    assert_eq!(aSurf.nb_v_poles(), 3);
    assert_eq!(aSurf.order_u(), 1);
    assert_eq!(aSurf.order_v(), 1);
    EXPECT_NEAR(aSurf.alpha_u(), 1.0, precision::confusion());
    EXPECT_NEAR(aSurf.alpha_v(), 1.0, precision::confusion());
    assert!(!aSurf.is_rational());
}

// Test construction throws for invalid parameters
#[test]
fn GeomEval_TBezierSurfaceTest__Construction_InvalidParams_Throws() {
    // Even number of poles in U (not odd)
    let mut aPoles = grid(2, 3);
    for i in 1..=2 {
        for j in 1..=3 {
            let aPole = Point3::new(f64::from(i), f64::from(j), 0.0);
            aPoles.set_value(i, j, aPole).unwrap();
        }
    }
    EXPECT_THROW(
        GeomEvalTBezierSurface::new(&aPoles, 1.0, 1.0),
        "Standard_ConstructionError",
    );
}

// Test parameter range
#[test]
fn GeomEval_TBezierSurfaceTest__ParameterRange() {
    let aSurf = createSimpleSurface();
    let (aU1, aU2, aV1, aV2) = aSurf.bounds();
    EXPECT_NEAR(aU1, 0.0, precision::confusion());
    EXPECT_NEAR(aU2, PI, precision::confusion()); // Pi/alphaU = Pi/1
    EXPECT_NEAR(aV1, 0.0, precision::confusion());
    EXPECT_NEAR(aV2, PI, precision::confusion()); // Pi/alphaV = Pi/1
}

// Test EvalD0 at corners
#[test]
fn GeomEval_TBezierSurfaceTest__EvalD0_Corners() {
    let aSurf = createSimpleSurface();
    let (aU1, aU2, aV1, aV2) = aSurf.bounds();

    // The corner evaluations should be well-defined points
    let aP00: Point3 = aSurf.eval_d0(aU1, aV1).unwrap();
    let aP10: Point3 = aSurf.eval_d0(aU2, aV1).unwrap();
    let aP01: Point3 = aSurf.eval_d0(aU1, aV2).unwrap();
    let aP11: Point3 = aSurf.eval_d0(aU2, aV2).unwrap();

    // Verify points are distinct (surface is not degenerate)
    assert!(aP00.distance(&aP10) > precision::confusion());
    assert!(aP00.distance(&aP01) > precision::confusion());
    assert!(aP00.distance(&aP11) > precision::confusion());
}

// Test derivative consistency: compare EvalD1 vs finite difference of EvalD0
#[test]
fn GeomEval_TBezierSurfaceTest__EvalD1_ConsistentWithD0() {
    let aSurf = createSimpleSurface();
    let (aU, aV) = (PI / 3.0, PI / 4.0);
    let aEps: f64 = precision::confusion();

    let aD1: GeomSurfaceD1 = aSurf.eval_d1(aU, aV).unwrap();
    let aPu1: Point3 = aSurf.eval_d0(aU + aEps, aV).unwrap();
    let aPu2: Point3 = aSurf.eval_d0(aU - aEps, aV).unwrap();
    let aPv1: Point3 = aSurf.eval_d0(aU, aV + aEps).unwrap();
    let aPv2: Point3 = aSurf.eval_d0(aU, aV - aEps).unwrap();

    let aFDU: Vector3 = central(aPu1.xyz(), aPu2.xyz());
    let aFDV: Vector3 = central(aPv1.xyz(), aPv2.xyz());

    expect_vec_near(aD1.d1u, aFDU, THE_FD_TOL_D1);
    expect_vec_near(aD1.d1v, aFDV, THE_FD_TOL_D1);
}

// Test periodicity
#[test]
fn GeomEval_TBezierSurfaceTest__Periodicity() {
    let aSurf = createSimpleSurface();
    assert!(!aSurf.is_u_periodic());
    assert!(!aSurf.is_v_periodic());
}

#[test]
fn GeomEval_TBezierSurfaceTest__Reverse_NotImplemented() {
    let aSurf = createSimpleSurface();
    EXPECT_THROW(aSurf.u_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.u_reversed_parameter(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reversed_parameter(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_TBezierSurfaceTest__Iso_NotImplemented() {
    let aSurf = createSimpleSurface();
    EXPECT_THROW(aSurf.u_iso(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_iso(0.5), "Standard_NotImplemented");
}

// Test Transform/Transformed are not implemented.
#[test]
fn GeomEval_TBezierSurfaceTest__Transform_NotImplemented() {
    let aSurf = createSimpleSurface();
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));

    EXPECT_THROW(aSurf.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.transformed(aTrsf), "Standard_NotImplemented");
}

// Test Copy produces independent identical object
#[test]
fn GeomEval_TBezierSurfaceTest__Copy_Independent() {
    let aSurf = createSimpleSurface();
    let aCopy: Handle<dyn GeomGeometry> = aSurf.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopySurf: Handle<GeomEvalTBezierSurface> = aCopy.cast();
    assert!(!aCopySurf.is_null());
    assert_eq!(aCopySurf.nb_u_poles(), 3);
    assert_eq!(aCopySurf.nb_v_poles(), 3);
}

// Test DumpJson does not crash
#[test]
fn GeomEval_TBezierSurfaceTest__DumpJson_NoCrash() {
    let aSurf = createSimpleSurface();
    let mut aSS = DumpStream::default();
    aSurf.dump_json(&mut aSS, -1).unwrap();
}

// Test EvalD2 components match finite differences of D1
#[test]
fn GeomEval_TBezierSurfaceTest__EvalD2_ConsistentWithD1() {
    let aSurf = createSimpleSurface();
    let (aU, aV) = (PI / 4.0, PI / 4.0);
    let aEps: f64 = precision::confusion();

    let aD2: GeomSurfaceD2 = aSurf.eval_d2(aU, aV).unwrap();

    // D2U: finite difference of D1U w.r.t. U
    let aD1Uplus: GeomSurfaceD1 = aSurf.eval_d1(aU + aEps, aV).unwrap();
    let aD1Uminus: GeomSurfaceD1 = aSurf.eval_d1(aU - aEps, aV).unwrap();
    let aFD_D2U: Vector3 = central(*aD1Uplus.d1u.xyz(), *aD1Uminus.d1u.xyz());
    expect_vec_near(aD2.d2u, aFD_D2U, THE_FD_TOL_D2);

    // D2V: finite difference of D1V w.r.t. V
    let aD1Vplus: GeomSurfaceD1 = aSurf.eval_d1(aU, aV + aEps).unwrap();
    let aD1Vminus: GeomSurfaceD1 = aSurf.eval_d1(aU, aV - aEps).unwrap();
    let aFD_D2V: Vector3 = central(*aD1Vplus.d1v.xyz(), *aD1Vminus.d1v.xyz());
    expect_vec_near(aD2.d2v, aFD_D2V, THE_FD_TOL_D2);

    // D2UV: finite difference of D1U w.r.t. V
    let aFD_D2UV: Vector3 = central(*aD1Vplus.d1u.xyz(), *aD1Vminus.d1u.xyz());
    expect_vec_near(aD2.d2uv, aFD_D2UV, THE_FD_TOL_D2);
}

// Test EvalD3 components match finite differences of D2
#[test]
fn GeomEval_TBezierSurfaceTest__EvalD3_ConsistentWithD2() {
    let aSurf = createSimpleSurface();
    let (aU, aV) = (PI / 3.0, PI / 3.0);
    let aEps: f64 = precision::confusion();

    let aD3: GeomSurfaceD3 = aSurf.eval_d3(aU, aV).unwrap();

    // D3U: finite difference of D2U w.r.t. U
    let aD2Uplus: GeomSurfaceD2 = aSurf.eval_d2(aU + aEps, aV).unwrap();
    let aD2Uminus: GeomSurfaceD2 = aSurf.eval_d2(aU - aEps, aV).unwrap();
    let aFD_D3U: Vector3 = central(*aD2Uplus.d2u.xyz(), *aD2Uminus.d2u.xyz());
    expect_vec_near(aD3.d3u, aFD_D3U, THE_FD_TOL_D2);

    // D3V: finite difference of D2V w.r.t. V
    let aD2Vplus: GeomSurfaceD2 = aSurf.eval_d2(aU, aV + aEps).unwrap();
    let aD2Vminus: GeomSurfaceD2 = aSurf.eval_d2(aU, aV - aEps).unwrap();
    let aFD_D3V: Vector3 = central(*aD2Vplus.d2v.xyz(), *aD2Vminus.d2v.xyz());
    expect_vec_near(aD3.d3v, aFD_D3V, THE_FD_TOL_D2);

    // D3UUV: finite difference of D2U w.r.t. V
    let aFD_D3UUV: Vector3 = central(*aD2Vplus.d2u.xyz(), *aD2Vminus.d2u.xyz());
    expect_vec_near(aD3.d3uuv, aFD_D3UUV, THE_FD_TOL_D2);

    // D3UVV: finite difference of D2V w.r.t. U
    let aFD_D3UVV: Vector3 = central(*aD2Uplus.d2v.xyz(), *aD2Uminus.d2v.xyz());
    expect_vec_near(aD3.d3uvv, aFD_D3UVV, THE_FD_TOL_D2);
}

// Test EvalDN(u,v,1,0) and EvalDN(u,v,0,1) match D1U and D1V from EvalD1
#[test]
fn GeomEval_TBezierSurfaceTest__EvalDN_ConsistentWithD1() {
    let aSurf = createSimpleSurface();
    let (aU, aV) = (PI / 4.0, PI / 3.0);

    let aD1: GeomSurfaceD1 = aSurf.eval_d1(aU, aV).unwrap();
    let aDN_U: Vector3 = aSurf.eval_dn(aU, aV, 1, 0).unwrap();
    let aDN_V: Vector3 = aSurf.eval_dn(aU, aV, 0, 1).unwrap();

    expect_vec_near(aD1.d1u, aDN_U, precision::confusion());
    expect_vec_near(aD1.d1v, aDN_V, precision::confusion());
}

// The unit sphere of gp_Ax3() that the sphere-patch tests compare against.
fn unit_sphere() -> Handle<GeomSphericalSurface> {
    Handle::new(GeomSphericalSurface::new(Axis3::default(), 1.0).unwrap())
}

// Test T-Bezier sphere patch matches Geom_SphericalSurface D0
#[test]
fn GeomEval_TBezierSurfaceTest__EvalD0_MatchesSphere() {
    let aTBez = createSpherePatch();
    let aSphere = unit_sphere();

    let aUParams = [
        0.1,
        PI / 6.0,
        PI / 4.0,
        PI / 3.0,
        PI / 2.0,
        2.0 * PI / 3.0,
        2.5,
    ];
    let aVParams = [0.1, PI / 6.0, PI / 4.0, PI / 3.0, 1.2];
    for aU in aUParams {
        for aV in aVParams {
            let aPT: Point3 = aTBez.eval_d0(aU, aV).unwrap();
            let aPS: Point3 = aSphere.eval_d0(aU, aV).unwrap();
            EXPECT_NEAR(aPT.distance(&aPS), 0.0, precision::angular());
        }
    }
}

// Test T-Bezier sphere patch matches Geom_SphericalSurface D1
#[test]
fn GeomEval_TBezierSurfaceTest__EvalD1_MatchesSphere() {
    let aTBez = createSpherePatch();
    let aSphere = unit_sphere();

    let aUParams = [PI / 6.0, PI / 3.0, PI / 2.0, 2.0 * PI / 3.0];
    let aVParams = [0.2, PI / 4.0, PI / 3.0, 1.0];
    for aU in aUParams {
        for aV in aVParams {
            let aDT: GeomSurfaceD1 = aTBez.eval_d1(aU, aV).unwrap();
            let aDS: GeomSurfaceD1 = aSphere.eval_d1(aU, aV).unwrap();

            EXPECT_NEAR(aDT.point.distance(&aDS.point), 0.0, precision::angular());
            expect_vec_near(aDT.d1u, aDS.d1u, precision::angular());
            expect_vec_near(aDT.d1v, aDS.d1v, precision::angular());
        }
    }
}

// Test T-Bezier sphere patch matches Geom_SphericalSurface D2
#[test]
fn GeomEval_TBezierSurfaceTest__EvalD2_MatchesSphere() {
    let aTBez = createSpherePatch();
    let aSphere = unit_sphere();

    let aUParams = [PI / 6.0, PI / 3.0, PI / 2.0];
    let aVParams = [0.3, PI / 4.0, 1.0];
    for aU in aUParams {
        for aV in aVParams {
            let aDT: GeomSurfaceD2 = aTBez.eval_d2(aU, aV).unwrap();
            let aDS: GeomSurfaceD2 = aSphere.eval_d2(aU, aV).unwrap();

            EXPECT_NEAR(aDT.point.distance(&aDS.point), 0.0, precision::angular());
            expect_vec_near(aDT.d1u, aDS.d1u, precision::angular());
            expect_vec_near(aDT.d1v, aDS.d1v, precision::angular());
            expect_vec_near(aDT.d2u, aDS.d2u, precision::angular());
            expect_vec_near(aDT.d2v, aDS.d2v, precision::angular());
            expect_vec_near(aDT.d2uv, aDS.d2uv, precision::angular());
        }
    }
}
