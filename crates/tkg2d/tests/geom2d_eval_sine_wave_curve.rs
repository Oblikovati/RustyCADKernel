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

//! Direct translation of Geom2dEval_SineWaveCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::Geom2dGeometry;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use rustycad_tkg2d::geom2d_eval::Geom2dEvalSineWaveCurve;
use rustycad_tkmath::gp::{Axis2d, Point2, Transform2, Vector2};
use std::f64::consts::PI;

const THE_FD_TOL: f64 = 1e-5;
const THE_FD_TOL_DN: f64 = 1e-4;

#[test]
fn Geom2dEval_SineWaveCurveTest__Construction_ValidParams() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.0).unwrap();
    EXPECT_NEAR(aCurve.amplitude(), 2.0, precision::confusion());
    EXPECT_NEAR(aCurve.omega(), 3.0, precision::confusion());
}

#[test]
fn Geom2dEval_SineWaveCurveTest__Construction_InvalidParams_Throws() {
    let anAx2d = Axis2d::default();
    EXPECT_THROW(
        Geom2dEvalSineWaveCurve::new(anAx2d, 0.0, 1.0, 0.0),
        "Standard_ConstructionError",
    );
    EXPECT_THROW(
        Geom2dEvalSineWaveCurve::new(anAx2d, 1.0, 0.0, 0.0),
        "Standard_ConstructionError",
    );
}

#[test]
fn Geom2dEval_SineWaveCurveTest__EvalD0_AtKnownPoints() {
    let anAx2d = Axis2d::default();
    let (aA, aOm) = (2.0, 3.0);
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, aA, aOm, 0.0).unwrap();

    // t=0: C(0) = (0, 0)
    let aP0: Point2 = aCurve.eval_d0(0.0).unwrap();
    EXPECT_NEAR(aP0.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP0.y(), 0.0, precision::confusion());

    // t = Pi/(2*omega): C = (Pi/(2*omega), A)
    let aT1: f64 = PI / (2.0 * aOm);
    let aP1: Point2 = aCurve.eval_d0(aT1).unwrap();
    EXPECT_NEAR(aP1.x(), aT1, precision::confusion());
    EXPECT_NEAR(aP1.y(), aA, precision::confusion());
}

#[test]
fn Geom2dEval_SineWaveCurveTest__IsPeriodic() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.0).unwrap();
    assert!(!aCurve.is_periodic());
    // The pinned oracle build does not define No_Exception.
    EXPECT_THROW(aCurve.period(), "Standard_NoSuchObject");
}

#[test]
fn Geom2dEval_SineWaveCurveTest__Reverse_NotImplemented() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.0).unwrap();
    EXPECT_THROW(aCurve.reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.reversed_parameter(0.5), "Standard_NotImplemented");
}

#[test]
fn Geom2dEval_SineWaveCurveTest__Transform_NotImplemented() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.0).unwrap();
    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(1.0, 2.0));
    EXPECT_THROW(aCurve.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.transformed(aTrsf), "Standard_NotImplemented");
}

#[test]
fn Geom2dEval_SineWaveCurveTest__EvalD1_ConsistentWithD0() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.5).unwrap();
    let aT: f64 = 1.0;

    let aD1: Geom2dCurveD1 = aCurve.eval_d1(aT).unwrap();
    let aP1: Point2 = aCurve.eval_d0(aT + precision::confusion()).unwrap();
    let aP2: Point2 = aCurve.eval_d0(aT - precision::confusion()).unwrap();
    let aFD = Vector2::from((aP1.xy() - aP2.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD1.d1.x(), aFD.x(), THE_FD_TOL);
    EXPECT_NEAR(aD1.d1.y(), aFD.y(), THE_FD_TOL);
}

#[test]
fn Geom2dEval_SineWaveCurveTest__EvalD2_ConsistentWithD1() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.5).unwrap();
    let aT: f64 = 1.0;

    let aD2: Geom2dCurveD2 = aCurve.eval_d2(aT).unwrap();
    let aD1p: Geom2dCurveD1 = aCurve.eval_d1(aT + precision::confusion()).unwrap();
    let aD1m: Geom2dCurveD1 = aCurve.eval_d1(aT - precision::confusion()).unwrap();
    let aFD = Vector2::from((aD1p.d1.xy() - aD1m.d1.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD2.d2.x(), aFD.x(), THE_FD_TOL);
    EXPECT_NEAR(aD2.d2.y(), aFD.y(), THE_FD_TOL);
}

#[test]
fn Geom2dEval_SineWaveCurveTest__EvalD3_ConsistentWithD2() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.5).unwrap();
    let aT: f64 = 1.0;

    let aD3: Geom2dCurveD3 = aCurve.eval_d3(aT).unwrap();
    let aD2p: Geom2dCurveD2 = aCurve.eval_d2(aT + precision::confusion()).unwrap();
    let aD2m: Geom2dCurveD2 = aCurve.eval_d2(aT - precision::confusion()).unwrap();
    let aFD = Vector2::from((aD2p.d2.xy() - aD2m.d2.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD3.d3.x(), aFD.x(), THE_FD_TOL);
    EXPECT_NEAR(aD3.d3.y(), aFD.y(), THE_FD_TOL);
}

#[test]
fn Geom2dEval_SineWaveCurveTest__EvalDN_HigherOrder_ConsistentWithPreviousOrder() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.5).unwrap();
    let aT: f64 = 1.2;
    let aN: i32 = 6;

    let aDN: Vector2 = aCurve.eval_dn(aT, aN).unwrap();
    let aDNm1P: Vector2 = aCurve.eval_dn(aT + precision::confusion(), aN - 1).unwrap();
    let aDNm1M: Vector2 = aCurve.eval_dn(aT - precision::confusion(), aN - 1).unwrap();
    let aFD = Vector2::from((aDNm1P.xy() - aDNm1M.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aDN.x(), aFD.x(), THE_FD_TOL_DN);
    EXPECT_NEAR(aDN.y(), aFD.y(), THE_FD_TOL_DN);
}

#[test]
fn Geom2dEval_SineWaveCurveTest__Copy_Independent() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.5).unwrap();
    let aCopy = aCurve.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyCurve: Handle<Geom2dEvalSineWaveCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());
    EXPECT_NEAR(aCopyCurve.amplitude(), 2.0, precision::confusion());
}

#[test]
fn Geom2dEval_SineWaveCurveTest__DumpJson_NoCrash() {
    let anAx2d = Axis2d::default();
    let aCurve = Geom2dEvalSineWaveCurve::new(anAx2d, 2.0, 3.0, 0.0).unwrap();
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}
