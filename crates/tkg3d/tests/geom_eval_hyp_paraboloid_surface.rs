// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Direct translation of GeomEval_HypParaboloidSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
#[path = "geom_eval_surface_support/mod.rs"]
mod support;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use rustycad_tkg3d::geom_eval::GeomEvalHypParaboloidSurface;
use rustycad_tkmath::gp::{Axis3, Point3, Transform3, Vector3};
use support::{EXPECT_VEC_NEAR, point_difference, quadric_residual};

const THE_FD_TOL: f64 = 1e-5;

fn saddle(a: f64, b: f64) -> GeomEvalHypParaboloidSurface {
    GeomEvalHypParaboloidSurface::new(Axis3::default(), a, b).unwrap()
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__Construction_ValidParams() {
    let aSurf = saddle(2.0, 3.0);
    EXPECT_NEAR(aSurf.semi_axis_a(), 2.0, precision::confusion());
    EXPECT_NEAR(aSurf.semi_axis_b(), 3.0, precision::confusion());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__Construction_InvalidAxes_Throws() {
    let anAx3 = Axis3::default();
    let build = |a: f64, b: f64| GeomEvalHypParaboloidSurface::new(anAx3, a, b);
    EXPECT_THROW(build(-1.0, 1.0), "Standard_ConstructionError");
    EXPECT_THROW(build(1.0, 0.0), "Standard_ConstructionError");
    EXPECT_THROW(build(0.0, 1.0), "Standard_ConstructionError");
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__EvalD0_Origin() {
    let aP: Point3 = saddle(2.0, 3.0).eval_d0(0.0, 0.0).unwrap();
    EXPECT_NEAR(aP.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.z(), 0.0, precision::confusion());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__EvalD0_KnownPoint_U() {
    let (aA, aB) = (2.0, 3.0);
    let aP: Point3 = saddle(aA, aB).eval_d0(1.0, 0.0).unwrap();
    EXPECT_NEAR(aP.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aP.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.z(), 1.0 / (aA * aA), precision::confusion());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__EvalD0_KnownPoint_V() {
    let (aA, aB) = (2.0, 3.0);
    let aP: Point3 = saddle(aA, aB).eval_d0(0.0, 1.0).unwrap();
    EXPECT_NEAR(aP.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.y(), 1.0, precision::confusion());
    EXPECT_NEAR(aP.z(), -1.0 / (aB * aB), precision::confusion());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__EvalD1_ConsistentWithD0() {
    let aSurf = saddle(2.0, 3.0);
    let (aU, aV) = (1.5, 0.7);
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
fn GeomEval_HypParaboloidSurfaceTest__EvalD2_ConstantZComponent() {
    let (aA, aB) = (2.0, 3.0);
    let aSurf = saddle(aA, aB);
    let aD2a: GeomSurfaceD2 = aSurf.eval_d2(0.0, 0.0).unwrap();
    let aD2b: GeomSurfaceD2 = aSurf.eval_d2(1.0, 2.0).unwrap();
    let aD2c: GeomSurfaceD2 = aSurf.eval_d2(-3.0, 5.0).unwrap();
    let aExpD2U_Z: f64 = 2.0 / (aA * aA);
    let aExpD2V_Z: f64 = -2.0 / (aB * aB);
    EXPECT_NEAR(aD2a.d2u.z(), aExpD2U_Z, precision::angular());
    EXPECT_NEAR(aD2b.d2u.z(), aExpD2U_Z, precision::angular());
    EXPECT_NEAR(aD2c.d2u.z(), aExpD2U_Z, precision::angular());
    EXPECT_NEAR(aD2a.d2v.z(), aExpD2V_Z, precision::angular());
    EXPECT_NEAR(aD2b.d2v.z(), aExpD2V_Z, precision::angular());
    EXPECT_NEAR(aD2c.d2v.z(), aExpD2V_Z, precision::angular());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__Bounds_Periodicity() {
    let aSurf = saddle(1.0, 1.0);
    let (aU1, aU2, aV1, aV2) = aSurf.bounds();
    assert!(aU1 < -1e10);
    assert!(aU2 > 1e10);
    assert!(aV1 < -1e10);
    assert!(aV2 > 1e10);
    assert!(!aSurf.is_u_periodic());
    assert!(!aSurf.is_u_closed().unwrap());
    assert!(!aSurf.is_v_periodic());
    assert!(!aSurf.is_v_closed().unwrap());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__Iso_NotImplemented() {
    let aSurf = saddle(2.0, 3.0);
    EXPECT_THROW(aSurf.u_iso(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_iso(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__Reverse_NotImplemented() {
    let aSurf = saddle(2.0, 3.0);
    EXPECT_THROW(aSurf.u_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.u_reversed_parameter(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reversed_parameter(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__Coefficients_SatisfiedAtEvalPoints() {
    let aSurf = saddle(2.0, 3.0);
    let aCoefficients: [f64; 10] = aSurf.coefficients();
    let mut u: f64 = -2.0;
    while u <= 2.0 {
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
fn GeomEval_HypParaboloidSurfaceTest__Transform_NotImplemented() {
    let aSurf = saddle(2.0, 3.0);
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));
    EXPECT_THROW(aSurf.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.transformed(aTrsf), "Standard_NotImplemented");
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__Copy_Independent() {
    let aSurf = saddle(2.0, 3.0);
    let aCopy = aSurf.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopySurf = aCopy.cast::<GeomEvalHypParaboloidSurface>();
    assert!(!aCopySurf.is_null());
    EXPECT_NEAR(aCopySurf.semi_axis_a(), 2.0, precision::confusion());
    EXPECT_NEAR(aCopySurf.semi_axis_b(), 3.0, precision::confusion());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__DumpJson_NoCrash() {
    let aSurf = saddle(2.0, 3.0);
    let mut aSS = DumpStream::default();
    assert!(aSurf.dump_json(&mut aSS, -1).is_ok());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__EvalD3_Zero() {
    let aD3: GeomSurfaceD3 = saddle(2.0, 3.0).eval_d3(1.0, 1.0).unwrap();
    assert!(aD3.d3u.magnitude() < precision::square_confusion());
    assert!(aD3.d3v.magnitude() < precision::square_confusion());
    assert!(aD3.d3uuv.magnitude() < precision::square_confusion());
    assert!(aD3.d3uvv.magnitude() < precision::square_confusion());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__EvalD2_CrossDerivative() {
    let aSurf = saddle(2.0, 3.0);
    let aTestPoints: [[f64; 2]; 3] = [[0.0, 0.0], [1.5, -2.0], [-3.0, 4.5]];
    for [aU, aV] in aTestPoints {
        let aD2: GeomSurfaceD2 = aSurf.eval_d2(aU, aV).unwrap();
        EXPECT_NEAR(aD2.d2uv.x(), 0.0, precision::square_confusion());
        EXPECT_NEAR(aD2.d2uv.y(), 0.0, precision::square_confusion());
        EXPECT_NEAR(aD2.d2uv.z(), 0.0, precision::square_confusion());
    }
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__EvalD2_KnownValues() {
    let (aA, aB) = (2.0, 3.0);
    let aD2: GeomSurfaceD2 = saddle(aA, aB).eval_d2(1.5, -2.7).unwrap();
    EXPECT_NEAR(aD2.d2u.x(), 0.0, precision::square_confusion());
    EXPECT_NEAR(aD2.d2u.y(), 0.0, precision::square_confusion());
    EXPECT_NEAR(aD2.d2u.z(), 2.0 / (aA * aA), precision::angular());
    EXPECT_NEAR(aD2.d2v.x(), 0.0, precision::square_confusion());
    EXPECT_NEAR(aD2.d2v.y(), 0.0, precision::square_confusion());
    EXPECT_NEAR(aD2.d2v.z(), -2.0 / (aB * aB), precision::angular());
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__EvalDN_HigherOrder() {
    let aSurf = saddle(2.0, 3.0);
    let (aU, aV) = (1.0, 1.0);
    let aD2: GeomSurfaceD2 = aSurf.eval_d2(aU, aV).unwrap();
    let aDN20: Vector3 = aSurf.eval_dn(aU, aV, 2, 0).unwrap();
    EXPECT_VEC_NEAR(aDN20, *aD2.d2u.xyz(), precision::confusion());
    let aDN02: Vector3 = aSurf.eval_dn(aU, aV, 0, 2).unwrap();
    EXPECT_VEC_NEAR(aDN02, *aD2.d2v.xyz(), precision::confusion());
    let aDN30: Vector3 = aSurf.eval_dn(aU, aV, 3, 0).unwrap();
    assert!(aDN30.magnitude() < precision::square_confusion());
}

// The bi-quadratic Bezier net of `(u, v, u^2/A^2 - v^2/B^2)` on [0, 1] x [0, 1].
fn bezier_saddle(aA: f64, aB: f64) -> Handle<GeomBezierSurface> {
    let aInvA2: f64 = 1.0 / (aA * aA);
    let aInvB2: f64 = 1.0 / (aB * aB);
    let mut aBPoles: Array2<'static, Point3> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        let aXi: f64 = 0.5 * f64::from(i - 1);
        let aZu: f64 = if i == 3 { aInvA2 } else { 0.0 };
        for j in 1..=3 {
            let aYj: f64 = 0.5 * f64::from(j - 1);
            let aZv: f64 = if j == 3 { -aInvB2 } else { 0.0 };
            aBPoles
                .set_value(i, j, Point3::new(aXi, aYj, aZu + aZv))
                .unwrap();
        }
    }
    Handle::new(GeomBezierSurface::new(&aBPoles).unwrap())
}

#[test]
fn GeomEval_HypParaboloidSurfaceTest__EvalD2_MatchesBezierSurface() {
    let (aA, aB) = (2.0, 3.0);
    let aSurf = saddle(aA, aB);
    let aBezier = bezier_saddle(aA, aB);
    let aParams: [[f64; 2]; 6] = [
        [0.0, 0.0],
        [0.25, 0.25],
        [0.5, 0.5],
        [0.75, 0.25],
        [0.3, 0.7],
        [1.0, 1.0],
    ];
    for [aU, aV] in aParams {
        let aD2H: GeomSurfaceD2 = aSurf.eval_d2(aU, aV).unwrap();
        let aD2B: GeomSurfaceD2 = aBezier.eval_d2(aU, aV).unwrap();
        EXPECT_NEAR(aD2H.point.distance(&aD2B.point), 0.0, precision::angular());
        EXPECT_VEC_NEAR(aD2H.d1u, *aD2B.d1u.xyz(), precision::angular());
        EXPECT_VEC_NEAR(aD2H.d1v, *aD2B.d1v.xyz(), precision::angular());
        EXPECT_VEC_NEAR(aD2H.d2u, *aD2B.d2u.xyz(), precision::angular());
        EXPECT_VEC_NEAR(aD2H.d2v, *aD2B.d2v.xyz(), precision::angular());
        EXPECT_VEC_NEAR(aD2H.d2uv, *aD2B.d2uv.xyz(), precision::angular());
    }
}
