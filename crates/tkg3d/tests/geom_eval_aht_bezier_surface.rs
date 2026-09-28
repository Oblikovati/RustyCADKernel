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

//! Direct translation of GeomEval_AHTBezierSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{
    GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use rustycad_tkg3d::geom_eval::GeomEvalAHTBezierSurface;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3, Xyz};

const THE_FD_TOL_D1: f64 = 1e-5;
const THE_FD_TOL_D2: f64 = 1e-4;

fn vecDiff(theV1: Vector3, theV2: Vector3) -> f64 {
    (theV1 - theV2).magnitude()
}

// The pole (i, j) of the test surfaces: (i - 1, j - 1, (i - 1) * (j - 1) * 0.1).
fn test_pole(i: i32, j: i32) -> Point3 {
    Point3::new(
        f64::from(i - 1),
        f64::from(j - 1),
        f64::from((i - 1) * (j - 1)) * 0.1,
    )
}

fn test_poles() -> Array2<'static, Point3> {
    let mut aPoles = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            aPoles.set_value(i, j, test_pole(i, j)).unwrap();
        }
    }
    aPoles
}

// Helper to create a pure polynomial AHT-Bezier surface.
// Basis: {1, t, t^2} in each direction => 3x3 poles, algDegree=2, alpha=0, beta=0
fn createPolynomialSurface() -> GeomEvalAHTBezierSurface {
    GeomEvalAHTBezierSurface::new(&test_poles(), [2, 2], [0.0, 0.0], [0.0, 0.0]).unwrap()
}

fn createPolynomialSurfaceRationalUniformWeight(theWeight: f64) -> GeomEvalAHTBezierSurface {
    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            aWeights.set_value(i, j, theWeight).unwrap();
        }
    }
    let aPoles = test_poles();
    GeomEvalAHTBezierSurface::with_weights(&aPoles, &aWeights, [2, 2], [0.0, 0.0], [0.0, 0.0])
        .unwrap()
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
fn GeomEval_AHTBezierSurfaceTest__Construction_ValidParams() {
    let aSurf = createPolynomialSurface();
    assert_eq!(aSurf.nb_poles_u(), 3);
    assert_eq!(aSurf.nb_poles_v(), 3);
    assert_eq!(aSurf.alg_degree_u(), 2);
    assert_eq!(aSurf.alg_degree_v(), 2);
    EXPECT_NEAR(aSurf.alpha_u(), 0.0, precision::confusion());
    EXPECT_NEAR(aSurf.alpha_v(), 0.0, precision::confusion());
    EXPECT_NEAR(aSurf.beta_u(), 0.0, precision::confusion());
    EXPECT_NEAR(aSurf.beta_v(), 0.0, precision::confusion());
}

// Test parameter range is [0, 1] x [0, 1]
#[test]
fn GeomEval_AHTBezierSurfaceTest__ParameterRange() {
    let aSurf = createPolynomialSurface();
    let (aU1, aU2, aV1, aV2) = aSurf.bounds();
    EXPECT_NEAR(aU1, 0.0, precision::confusion());
    EXPECT_NEAR(aU2, 1.0, precision::confusion());
    EXPECT_NEAR(aV1, 0.0, precision::confusion());
    EXPECT_NEAR(aV2, 1.0, precision::confusion());
}

// Test EvalD0 at corners
#[test]
fn GeomEval_AHTBezierSurfaceTest__EvalD0_Corners() {
    let aSurf = createPolynomialSurface();

    let aP00: Point3 = aSurf.eval_d0(0.0, 0.0).unwrap();
    let aP10: Point3 = aSurf.eval_d0(1.0, 0.0).unwrap();
    let aP01: Point3 = aSurf.eval_d0(0.0, 1.0).unwrap();
    let aP11: Point3 = aSurf.eval_d0(1.0, 1.0).unwrap();

    // Verify corners are distinct (non-degenerate surface)
    assert!(aP00.distance(&aP10) > precision::confusion());
    assert!(aP00.distance(&aP01) > precision::confusion());
    assert!(aP00.distance(&aP11) > precision::confusion());
}

// Test derivative consistency: compare EvalD1 vs finite difference of EvalD0
#[test]
fn GeomEval_AHTBezierSurfaceTest__EvalD1_ConsistentWithD0() {
    let aSurf = createPolynomialSurface();
    let (aU, aV) = (0.3, 0.7);
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
fn GeomEval_AHTBezierSurfaceTest__Periodicity() {
    let aSurf = createPolynomialSurface();
    assert!(!aSurf.is_u_periodic());
    assert!(!aSurf.is_v_periodic());
    assert!(!aSurf.is_u_closed().unwrap());
    assert!(!aSurf.is_v_closed().unwrap());
}

#[test]
fn GeomEval_AHTBezierSurfaceTest__Reverse_NotImplemented() {
    let aSurf = createPolynomialSurface();
    EXPECT_THROW(aSurf.u_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.u_reversed_parameter(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reversed_parameter(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_AHTBezierSurfaceTest__Iso_NotImplemented() {
    let aSurf = createPolynomialSurface();
    EXPECT_THROW(aSurf.u_iso(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_iso(0.5), "Standard_NotImplemented");
}

// Test Transform/Transformed are not implemented.
#[test]
fn GeomEval_AHTBezierSurfaceTest__Transform_NotImplemented() {
    let aSurf = createPolynomialSurface();
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));

    EXPECT_THROW(aSurf.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.transformed(aTrsf), "Standard_NotImplemented");
}

// Test Copy produces independent identical object
#[test]
fn GeomEval_AHTBezierSurfaceTest__Copy_Independent() {
    let aSurf = createPolynomialSurface();
    let aCopy: Handle<dyn GeomGeometry> = aSurf.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopySurf: Handle<GeomEvalAHTBezierSurface> = aCopy.cast();
    assert!(!aCopySurf.is_null());
    assert_eq!(aCopySurf.nb_poles_u(), 3);
    assert_eq!(aCopySurf.nb_poles_v(), 3);
    assert_eq!(aCopySurf.alg_degree_u(), 2);
}

// Test DumpJson does not crash
#[test]
fn GeomEval_AHTBezierSurfaceTest__DumpJson_NoCrash() {
    let aSurf = createPolynomialSurface();
    let mut aSS = DumpStream::default();
    aSurf.dump_json(&mut aSS, -1).unwrap();
}

// Test EvalD2 components match finite differences of D1
#[test]
fn GeomEval_AHTBezierSurfaceTest__EvalD2_ConsistentWithD1() {
    let aSurf = createPolynomialSurface();
    let (aU, aV) = (0.3, 0.4);
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
fn GeomEval_AHTBezierSurfaceTest__EvalD3_ConsistentWithD2() {
    let aSurf = createPolynomialSurface();
    let (aU, aV) = (0.4, 0.3);
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
fn GeomEval_AHTBezierSurfaceTest__EvalDN_ConsistentWithD1() {
    let aSurf = createPolynomialSurface();
    let (aU, aV) = (0.5, 0.5);

    let aD1: GeomSurfaceD1 = aSurf.eval_d1(aU, aV).unwrap();
    let aDN_U: Vector3 = aSurf.eval_dn(aU, aV, 1, 0).unwrap();
    let aDN_V: Vector3 = aSurf.eval_dn(aU, aV, 0, 1).unwrap();

    expect_vec_near(aD1.d1u, aDN_U, precision::confusion());
    expect_vec_near(aD1.d1v, aDN_V, precision::confusion());
}

#[test]
fn GeomEval_AHTBezierSurfaceTest__RationalUniformScale_Invariant() {
    let aRatW1 = createPolynomialSurfaceRationalUniformWeight(1.0);
    let aRatW5 = createPolynomialSurfaceRationalUniformWeight(5.0);
    let (aU, aV) = (0.37, 0.62);

    let aP1: Point3 = aRatW1.eval_d0(aU, aV).unwrap();
    EXPECT_NEAR(aP1.distance(&aRatW5.eval_d0(aU, aV).unwrap()), 0.0, 1e-12);

    let aD1W1: GeomSurfaceD1 = aRatW1.eval_d1(aU, aV).unwrap();
    let aD1W5: GeomSurfaceD1 = aRatW5.eval_d1(aU, aV).unwrap();
    EXPECT_NEAR(vecDiff(aD1W1.d1u, aD1W5.d1u), 0.0, 1e-12);
    EXPECT_NEAR(vecDiff(aD1W1.d1v, aD1W5.d1v), 0.0, 1e-12);

    let aD2W1: GeomSurfaceD2 = aRatW1.eval_d2(aU, aV).unwrap();
    let aD2W5: GeomSurfaceD2 = aRatW5.eval_d2(aU, aV).unwrap();
    EXPECT_NEAR(vecDiff(aD2W1.d2u, aD2W5.d2u), 0.0, 1e-12);
    EXPECT_NEAR(vecDiff(aD2W1.d2v, aD2W5.d2v), 0.0, 1e-12);
    EXPECT_NEAR(vecDiff(aD2W1.d2uv, aD2W5.d2uv), 0.0, 1e-12);

    let aD3W1: GeomSurfaceD3 = aRatW1.eval_d3(aU, aV).unwrap();
    let aD3W5: GeomSurfaceD3 = aRatW5.eval_d3(aU, aV).unwrap();
    EXPECT_NEAR(vecDiff(aD3W1.d3u, aD3W5.d3u), 0.0, 1e-12);
    EXPECT_NEAR(vecDiff(aD3W1.d3v, aD3W5.d3v), 0.0, 1e-12);
    EXPECT_NEAR(vecDiff(aD3W1.d3uuv, aD3W5.d3uuv), 0.0, 1e-12);
    EXPECT_NEAR(vecDiff(aD3W1.d3uvv, aD3W5.d3uvv), 0.0, 1e-12);

    let aDN1: Vector3 = aRatW1.eval_dn(aU, aV, 2, 1).unwrap();
    let aDN5: Vector3 = aRatW5.eval_dn(aU, aV, 2, 1).unwrap();
    EXPECT_NEAR(vecDiff(aDN1, aDN5), 0.0, 1e-12);
}
