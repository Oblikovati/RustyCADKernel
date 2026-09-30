//! Direct translation of GeomEval_ParaboloidSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
#[path = "geom_eval_surface_support/mod.rs"]
mod support;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{
    GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use rustycad_tkg3d::geom_eval::GeomEvalParaboloidSurface;
use rustycad_tkmath::gp::{Axis3, Point3, Transform3, Vector3};
use std::f64::consts::PI;
use support::{EXPECT_VEC_NEAR, point_difference, quadric_residual, vector_difference};

const THE_FD_TOL: f64 = 1e-5;
const THE_FD_TOL_D3: f64 = 1e-4;

fn paraboloid(focal: f64) -> GeomEvalParaboloidSurface {
    GeomEvalParaboloidSurface::new(Axis3::default(), focal).unwrap()
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__Construction_ValidParams() {
    EXPECT_NEAR(paraboloid(1.0).focal(), 1.0, precision::confusion());
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__Construction_InvalidFocal_Throws() {
    let anAx3 = Axis3::default();
    let error = "Standard_ConstructionError";
    EXPECT_THROW(GeomEvalParaboloidSurface::new(anAx3, -1.0), error);
    EXPECT_THROW(GeomEvalParaboloidSurface::new(anAx3, 0.0), error);
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__EvalD0_Origin() {
    let aP: Point3 = paraboloid(1.0).eval_d0(0.0, 0.0).unwrap();
    EXPECT_NEAR(aP.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.z(), 0.0, precision::confusion());
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__EvalD0_KnownPoint() {
    let aP: Point3 = paraboloid(1.0).eval_d0(0.0, 2.0).unwrap();
    EXPECT_NEAR(aP.x(), 2.0, precision::confusion());
    EXPECT_NEAR(aP.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.z(), 1.0, precision::confusion());
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__EvalD1_ConsistentWithD0() {
    let aSurf = paraboloid(2.0);
    let (aU, aV) = (1.0, 1.5);
    let h: f64 = precision::confusion();
    let aD1: GeomSurfaceD1 = aSurf.eval_d1(aU, aV).unwrap();
    let aPu1 = aSurf.eval_d0(aU + h, aV).unwrap();
    let aPu2 = aSurf.eval_d0(aU - h, aV).unwrap();
    let aPv1 = aSurf.eval_d0(aU, aV + h).unwrap();
    let aPv2 = aSurf.eval_d0(aU, aV - h).unwrap();
    EXPECT_VEC_NEAR(aD1.d1u, point_difference(aPu1, aPu2), THE_FD_TOL);
    EXPECT_VEC_NEAR(aD1.d1v, point_difference(aPv1, aPv2), THE_FD_TOL);
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__Bounds_Periodicity() {
    let aSurf = paraboloid(1.0);
    let (aU1, aU2, _aV1, _aV2) = aSurf.bounds();
    EXPECT_NEAR(aU1, 0.0, precision::confusion());
    EXPECT_NEAR(aU2, 2.0 * PI, precision::confusion());
    assert!(aSurf.is_u_periodic());
    assert!(aSurf.is_u_closed().unwrap());
    assert!(!aSurf.is_v_periodic());
    assert!(!aSurf.is_v_closed().unwrap());
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__Iso_NotImplemented() {
    let aSurf = paraboloid(1.0);
    EXPECT_THROW(aSurf.u_iso(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_iso(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__Reverse_NotImplemented() {
    let aSurf = paraboloid(1.0);
    EXPECT_THROW(aSurf.u_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.u_reversed_parameter(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reversed_parameter(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__Coefficients_SatisfiedAtEvalPoints() {
    let aSurf = paraboloid(1.5);
    let aCoefficients: [f64; 10] = aSurf.coefficients();
    let mut u: f64 = 0.0;
    while u < 6.0 {
        let mut v: f64 = -2.0;
        while v <= 2.0 {
            let aP = aSurf.eval_d0(u, v).unwrap();
            let aVal: f64 = quadric_residual(aCoefficients, aP);
            EXPECT_NEAR(aVal, 0.0, precision::intersection());
            v += 1.0;
        }
        u += 1.0;
    }
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__Transform_NotImplemented() {
    let aSurf = paraboloid(1.0);
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));
    EXPECT_THROW(aSurf.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.transformed(aTrsf), "Standard_NotImplemented");
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__Copy_Independent() {
    let aSurf = paraboloid(2.0);
    let aCopy = aSurf.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopySurf = aCopy.cast::<GeomEvalParaboloidSurface>();
    assert!(!aCopySurf.is_null());
    EXPECT_NEAR(aCopySurf.focal(), 2.0, precision::confusion());
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__DumpJson_NoCrash() {
    let aSurf = paraboloid(1.0);
    let mut aSS = DumpStream::default();
    assert!(aSurf.dump_json(&mut aSS, -1).is_ok());
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__EvalD2_ConsistentWithD1() {
    let aSurf = paraboloid(2.0);
    let (aU, aV) = (1.0, 1.5);
    let h: f64 = precision::confusion();
    let aD2: GeomSurfaceD2 = aSurf.eval_d2(aU, aV).unwrap();
    let aD1Uf = aSurf.eval_d1(aU + h, aV).unwrap();
    let aD1Ub = aSurf.eval_d1(aU - h, aV).unwrap();
    let aD1Vf = aSurf.eval_d1(aU, aV + h).unwrap();
    let aD1Vb = aSurf.eval_d1(aU, aV - h).unwrap();
    EXPECT_VEC_NEAR(aD2.d2u, vector_difference(aD1Uf.d1u, aD1Ub.d1u), THE_FD_TOL);
    EXPECT_VEC_NEAR(aD2.d2v, vector_difference(aD1Vf.d1v, aD1Vb.d1v), THE_FD_TOL);
    EXPECT_VEC_NEAR(
        aD2.d2uv,
        vector_difference(aD1Vf.d1u, aD1Vb.d1u),
        THE_FD_TOL,
    );
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__EvalD3_ConsistentWithD2() {
    let aSurf = paraboloid(2.0);
    let (aU, aV) = (0.7, 1.2);
    let h: f64 = precision::confusion();
    let aD3: GeomSurfaceD3 = aSurf.eval_d3(aU, aV).unwrap();
    let aD2Uf = aSurf.eval_d2(aU + h, aV).unwrap();
    let aD2Ub = aSurf.eval_d2(aU - h, aV).unwrap();
    let aD2Vf = aSurf.eval_d2(aU, aV + h).unwrap();
    let aD2Vb = aSurf.eval_d2(aU, aV - h).unwrap();
    EXPECT_VEC_NEAR(
        aD3.d3u,
        vector_difference(aD2Uf.d2u, aD2Ub.d2u),
        THE_FD_TOL_D3,
    );
    EXPECT_VEC_NEAR(
        aD3.d3v,
        vector_difference(aD2Vf.d2v, aD2Vb.d2v),
        THE_FD_TOL_D3,
    );
    EXPECT_VEC_NEAR(
        aD3.d3uuv,
        vector_difference(aD2Vf.d2u, aD2Vb.d2u),
        THE_FD_TOL_D3,
    );
    EXPECT_VEC_NEAR(
        aD3.d3uvv,
        vector_difference(aD2Uf.d2v, aD2Ub.d2v),
        THE_FD_TOL_D3,
    );
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__EvalD2_KnownAnalytical() {
    let aFocal: f64 = 2.0;
    let (aU, aV) = (PI / 4.0, 2.0);
    let aD2: GeomSurfaceD2 = paraboloid(aFocal).eval_d2(aU, aV).unwrap();
    let aExpX: f64 = -aV * aU.cos();
    let aExpY: f64 = -aV * aU.sin();
    let aExpZ: f64 = 0.0;
    EXPECT_NEAR(aD2.d2u.x(), aExpX, precision::angular());
    EXPECT_NEAR(aD2.d2u.y(), aExpY, precision::angular());
    EXPECT_NEAR(aD2.d2u.z(), aExpZ, precision::angular());
    EXPECT_NEAR(aD2.d2u.magnitude(), aV, precision::angular());
}

#[test]
fn GeomEval_ParaboloidSurfaceTest__EvalDN_Consistency() {
    let aSurf = paraboloid(2.0);
    let (aU, aV) = (1.0, 1.0);
    let aD1: GeomSurfaceD1 = aSurf.eval_d1(aU, aV).unwrap();
    let aDN10: Vector3 = aSurf.eval_dn(aU, aV, 1, 0).unwrap();
    let aDN01: Vector3 = aSurf.eval_dn(aU, aV, 0, 1).unwrap();
    EXPECT_VEC_NEAR(aDN10, *aD1.d1u.xyz(), precision::confusion());
    EXPECT_VEC_NEAR(aDN01, *aD1.d1v.xyz(), precision::confusion());
}
