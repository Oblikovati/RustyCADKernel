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

//! Direct translation of GeomEval_EllipsoidSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
#[path = "geom_eval_surface_support/mod.rs"]
mod support;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{
    GeomGeometry, GeomSphericalSurface, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use rustycad_tkg3d::geom_eval::GeomEvalEllipsoidSurface;
use rustycad_tkmath::gp::{Axis3, Point3, Transform3, Vector3};
use std::f64::consts::PI;
use support::{EXPECT_VEC_NEAR, point_difference, quadric_residual, vector_difference};

const THE_FD_TOL: f64 = 1e-5;

fn ellipsoid(a: f64, b: f64, c: f64) -> GeomEvalEllipsoidSurface {
    GeomEvalEllipsoidSurface::new(Axis3::default(), a, b, c).unwrap()
}

fn sphere(radius: f64) -> Handle<GeomSphericalSurface> {
    Handle::new(GeomSphericalSurface::new(Axis3::default(), radius).unwrap())
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__Construction_ValidParams() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    EXPECT_NEAR(aSurf.semi_axis_a(), 3.0, precision::confusion());
    EXPECT_NEAR(aSurf.semi_axis_b(), 2.0, precision::confusion());
    EXPECT_NEAR(aSurf.semi_axis_c(), 1.0, precision::confusion());
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__Construction_InvalidAxes_Throws() {
    let anAx3 = Axis3::default();
    let error = "Standard_ConstructionError";
    EXPECT_THROW(GeomEvalEllipsoidSurface::new(anAx3, 0.0, 1.0, 1.0), error);
    EXPECT_THROW(GeomEvalEllipsoidSurface::new(anAx3, 1.0, -1.0, 1.0), error);
    EXPECT_THROW(GeomEvalEllipsoidSurface::new(anAx3, 1.0, 1.0, 0.0), error);
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD0_XAxis() {
    let (aA, aB, aC) = (3.0, 2.0, 1.0);
    let aP: Point3 = ellipsoid(aA, aB, aC).eval_d0(0.0, 0.0).unwrap();
    EXPECT_NEAR(aP.x(), aA, precision::confusion());
    EXPECT_NEAR(aP.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.z(), 0.0, precision::confusion());
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD0_YAxis() {
    let (aA, aB, aC) = (3.0, 2.0, 1.0);
    let aP: Point3 = ellipsoid(aA, aB, aC).eval_d0(PI / 2.0, 0.0).unwrap();
    EXPECT_NEAR(aP.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.y(), aB, precision::confusion());
    EXPECT_NEAR(aP.z(), 0.0, precision::confusion());
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD0_ZAxis() {
    let (aA, aB, aC) = (3.0, 2.0, 1.0);
    let aP: Point3 = ellipsoid(aA, aB, aC).eval_d0(0.0, PI / 2.0).unwrap();
    EXPECT_NEAR(aP.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP.z(), aC, precision::confusion());
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD1_ConsistentWithD0() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    let (aU, aV) = (1.0, 0.5);
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
fn GeomEval_EllipsoidSurfaceTest__Bounds_Periodicity() {
    let aSurf = ellipsoid(1.0, 1.0, 1.0);
    let (aU1, aU2, aV1, aV2) = aSurf.bounds();
    EXPECT_NEAR(aU1, 0.0, precision::confusion());
    EXPECT_NEAR(aU2, 2.0 * PI, precision::confusion());
    EXPECT_NEAR(aV1, -PI / 2.0, precision::confusion());
    EXPECT_NEAR(aV2, PI / 2.0, precision::confusion());
    assert!(aSurf.is_u_periodic());
    assert!(aSurf.is_u_closed().unwrap());
    assert!(!aSurf.is_v_periodic());
    assert!(!aSurf.is_v_closed().unwrap());
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__Iso_NotImplemented() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    EXPECT_THROW(aSurf.u_iso(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_iso(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__Reverse_NotImplemented() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    EXPECT_THROW(aSurf.u_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.u_reversed_parameter(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reversed_parameter(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__Coefficients_SatisfiedAtEvalPoints() {
    let aSurf = ellipsoid(3.0, 2.0, 1.5);
    let aCoefficients: [f64; 10] = aSurf.coefficients();
    let mut u: f64 = 0.0;
    while u < 6.0 {
        let mut v: f64 = -1.0;
        while v <= 1.0 {
            let aP = aSurf.eval_d0(u, v).unwrap();
            let aVal: f64 = quadric_residual(aCoefficients, aP);
            EXPECT_NEAR(aVal, 0.0, precision::intersection());
            v += 0.5;
        }
        u += 1.0;
    }
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__Transform_NotImplemented() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));
    EXPECT_THROW(aSurf.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.transformed(aTrsf), "Standard_NotImplemented");
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__Copy_Independent() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    let aCopy = aSurf.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopySurf = aCopy.cast::<GeomEvalEllipsoidSurface>();
    assert!(!aCopySurf.is_null());
    EXPECT_NEAR(aCopySurf.semi_axis_a(), 3.0, precision::confusion());
    EXPECT_NEAR(aCopySurf.semi_axis_b(), 2.0, precision::confusion());
    EXPECT_NEAR(aCopySurf.semi_axis_c(), 1.0, precision::confusion());
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__DumpJson_NoCrash() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    let mut aSS = DumpStream::default();
    assert!(aSurf.dump_json(&mut aSS, -1).is_ok());
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD0_MatchesSphere() {
    let aR: f64 = 5.0;
    let (anEllipsoid, aSphere) = (ellipsoid(aR, aR, aR), sphere(aR));
    let aParams: [[f64; 2]; 6] = [
        [0.0, 0.0],
        [PI / 4.0, PI / 4.0],
        [PI / 2.0, 0.0],
        [PI, -PI / 4.0],
        [3.0 * PI / 2.0, 0.0],
        [1.0, 0.3],
    ];
    for [aU, aV] in aParams {
        let aPE = anEllipsoid.eval_d0(aU, aV).unwrap();
        let aPS = aSphere.eval_d0(aU, aV).unwrap();
        EXPECT_NEAR(aPE.x(), aPS.x(), precision::confusion());
        EXPECT_NEAR(aPE.y(), aPS.y(), precision::confusion());
        EXPECT_NEAR(aPE.z(), aPS.z(), precision::confusion());
    }
}

// The point with the Confusion tolerance, then each vector with the Angular one.
fn expect_matches(point: [Point3; 2], vectors: &[(Vector3, Vector3)]) {
    EXPECT_NEAR(point[0].x(), point[1].x(), precision::confusion());
    EXPECT_NEAR(point[0].y(), point[1].y(), precision::confusion());
    EXPECT_NEAR(point[0].z(), point[1].z(), precision::confusion());
    for (actual, expected) in vectors {
        EXPECT_VEC_NEAR(*actual, *expected.xyz(), precision::angular());
    }
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD1_MatchesSphere() {
    let aR: f64 = 5.0;
    let (anEllipsoid, aSphere) = (ellipsoid(aR, aR, aR), sphere(aR));
    let aParams: [[f64; 2]; 4] = [
        [0.0, 0.0],
        [PI / 4.0, PI / 4.0],
        [1.0, 0.3],
        [PI, -PI / 4.0],
    ];
    for [aU, aV] in aParams {
        let aDE: GeomSurfaceD1 = anEllipsoid.eval_d1(aU, aV).unwrap();
        let aDS: GeomSurfaceD1 = aSphere.eval_d1(aU, aV).unwrap();
        expect_matches(
            [aDE.point, aDS.point],
            &[(aDE.d1u, aDS.d1u), (aDE.d1v, aDS.d1v)],
        );
    }
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD2_MatchesSphere() {
    let aR: f64 = 5.0;
    let (anEllipsoid, aSphere) = (ellipsoid(aR, aR, aR), sphere(aR));
    let aParams: [[f64; 2]; 3] = [[0.0, 0.0], [PI / 4.0, PI / 4.0], [1.0, 0.3]];
    for [aU, aV] in aParams {
        let aDE: GeomSurfaceD2 = anEllipsoid.eval_d2(aU, aV).unwrap();
        let aDS: GeomSurfaceD2 = aSphere.eval_d2(aU, aV).unwrap();
        let vectors = [
            (aDE.d1u, aDS.d1u),
            (aDE.d1v, aDS.d1v),
            (aDE.d2u, aDS.d2u),
            (aDE.d2v, aDS.d2v),
            (aDE.d2uv, aDS.d2uv),
        ];
        expect_matches([aDE.point, aDS.point], &vectors);
    }
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD3_MatchesSphere() {
    let aR: f64 = 5.0;
    let (anEllipsoid, aSphere) = (ellipsoid(aR, aR, aR), sphere(aR));
    let aParams: [[f64; 2]; 2] = [[PI / 4.0, PI / 4.0], [1.0, 0.3]];
    for [aU, aV] in aParams {
        let aDE: GeomSurfaceD3 = anEllipsoid.eval_d3(aU, aV).unwrap();
        let aDS: GeomSurfaceD3 = aSphere.eval_d3(aU, aV).unwrap();
        let vectors = [
            (aDE.d1u, aDS.d1u),
            (aDE.d1v, aDS.d1v),
            (aDE.d2u, aDS.d2u),
            (aDE.d2v, aDS.d2v),
            (aDE.d2uv, aDS.d2uv),
            (aDE.d3u, aDS.d3u),
            (aDE.d3v, aDS.d3v),
            (aDE.d3uuv, aDS.d3uuv),
            (aDE.d3uvv, aDS.d3uvv),
        ];
        expect_matches([aDE.point, aDS.point], &vectors);
    }
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD2_ConsistentWithD1() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    let (aU, aV) = (1.0, 0.5);
    let h: f64 = precision::confusion();
    let aD2: GeomSurfaceD2 = aSurf.eval_d2(aU, aV).unwrap();
    let aD1Up = aSurf.eval_d1(aU + h, aV).unwrap();
    let aD1Um = aSurf.eval_d1(aU - h, aV).unwrap();
    EXPECT_VEC_NEAR(aD2.d2u, vector_difference(aD1Up.d1u, aD1Um.d1u), THE_FD_TOL);
    let aD1Vp = aSurf.eval_d1(aU, aV + h).unwrap();
    let aD1Vm = aSurf.eval_d1(aU, aV - h).unwrap();
    EXPECT_VEC_NEAR(aD2.d2v, vector_difference(aD1Vp.d1v, aD1Vm.d1v), THE_FD_TOL);
    EXPECT_VEC_NEAR(
        aD2.d2uv,
        vector_difference(aD1Vp.d1u, aD1Vm.d1u),
        THE_FD_TOL,
    );
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalD3_ConsistentWithD2() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    let (aU, aV) = (1.0, 0.5);
    let h: f64 = precision::confusion();
    let aD3: GeomSurfaceD3 = aSurf.eval_d3(aU, aV).unwrap();
    let aD2Up = aSurf.eval_d2(aU + h, aV).unwrap();
    let aD2Um = aSurf.eval_d2(aU - h, aV).unwrap();
    EXPECT_VEC_NEAR(aD3.d3u, vector_difference(aD2Up.d2u, aD2Um.d2u), THE_FD_TOL);
    let aD2Vp = aSurf.eval_d2(aU, aV + h).unwrap();
    let aD2Vm = aSurf.eval_d2(aU, aV - h).unwrap();
    EXPECT_VEC_NEAR(aD3.d3v, vector_difference(aD2Vp.d2v, aD2Vm.d2v), THE_FD_TOL);
    EXPECT_VEC_NEAR(
        aD3.d3uuv,
        vector_difference(aD2Vp.d2u, aD2Vm.d2u),
        THE_FD_TOL,
    );
    EXPECT_VEC_NEAR(
        aD3.d3uvv,
        vector_difference(aD2Up.d2v, aD2Um.d2v),
        THE_FD_TOL,
    );
}

#[test]
fn GeomEval_EllipsoidSurfaceTest__EvalDN_ConsistentWithD3() {
    let aSurf = ellipsoid(3.0, 2.0, 1.0);
    let (aU, aV) = (1.0, 0.5);
    let aD1: GeomSurfaceD1 = aSurf.eval_d1(aU, aV).unwrap();
    let aDN10: Vector3 = aSurf.eval_dn(aU, aV, 1, 0).unwrap();
    EXPECT_VEC_NEAR(aDN10, *aD1.d1u.xyz(), precision::angular());
    let aDN01: Vector3 = aSurf.eval_dn(aU, aV, 0, 1).unwrap();
    EXPECT_VEC_NEAR(aDN01, *aD1.d1v.xyz(), precision::angular());
}
