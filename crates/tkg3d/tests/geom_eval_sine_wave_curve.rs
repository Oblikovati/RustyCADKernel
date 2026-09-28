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

//! Direct translation of GeomEval_SineWaveCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomGeometry};
use rustycad_tkg3d::geom_eval::GeomEvalSineWaveCurve;
use rustycad_tkmath::gp::{Axis2, Point3, Transform3, Vector3};
use std::f64::consts::PI;

const THE_FD_TOL: f64 = 1e-5;

#[test]
fn GeomEval_SineWaveCurveTest__Construction_ValidParams() {
    let anAx2 = Axis2::default();
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, 2.0, 3.0, 0.0).unwrap();
    EXPECT_NEAR(aCurve.amplitude(), 2.0, precision::confusion());
    EXPECT_NEAR(aCurve.omega(), 3.0, precision::confusion());
    EXPECT_NEAR(aCurve.phase(), 0.0, precision::confusion());
}

#[test]
fn GeomEval_SineWaveCurveTest__Construction_InvalidParams_Throws() {
    let anAx2 = Axis2::default();
    // The native Phase argument defaults to 0.
    EXPECT_THROW(
        GeomEvalSineWaveCurve::new(anAx2, 0.0, 1.0, 0.0),
        "Standard_ConstructionError",
    );
    EXPECT_THROW(
        GeomEvalSineWaveCurve::new(anAx2, -1.0, 1.0, 0.0),
        "Standard_ConstructionError",
    );
    EXPECT_THROW(
        GeomEvalSineWaveCurve::new(anAx2, 1.0, 0.0, 0.0),
        "Standard_ConstructionError",
    );
    EXPECT_THROW(
        GeomEvalSineWaveCurve::new(anAx2, 1.0, -1.0, 0.0),
        "Standard_ConstructionError",
    );
}

#[test]
fn GeomEval_SineWaveCurveTest__EvalD0_AtKnownPoints() {
    let anAx2 = Axis2::default();
    let (aA, aOm) = (2.0, 3.0);
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, aA, aOm, 0.0).unwrap();

    // t=0: C(0) = (0, 0, 0) since sin(0)=0
    let aP0: Point3 = aCurve.eval_d0(0.0).unwrap();
    EXPECT_NEAR(aP0.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP0.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP0.z(), 0.0, precision::confusion());

    // t = Pi/(2*omega): C = (Pi/(2*omega), A, 0) since sin(Pi/2)=1
    let aT1: f64 = PI / (2.0 * aOm);
    let aP1: Point3 = aCurve.eval_d0(aT1).unwrap();
    EXPECT_NEAR(aP1.x(), aT1, precision::confusion());
    EXPECT_NEAR(aP1.y(), aA, precision::confusion());
    EXPECT_NEAR(aP1.z(), 0.0, precision::confusion());
}

#[test]
fn GeomEval_SineWaveCurveTest__IsPeriodic() {
    let anAx2 = Axis2::default();
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, 2.0, 3.0, 0.0).unwrap();
    assert!(!aCurve.is_periodic());
    // The pinned oracle build does not define No_Exception.
    EXPECT_THROW(aCurve.period(), "Standard_NoSuchObject");
    assert!(!aCurve.is_closed().unwrap());
}

#[test]
fn GeomEval_SineWaveCurveTest__Reverse_NotImplemented() {
    let anAx2 = Axis2::default();
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, 2.0, 3.0, 0.0).unwrap();
    EXPECT_THROW(aCurve.reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.reversed_parameter(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_SineWaveCurveTest__EvalD1_ConsistentWithD0() {
    let anAx2 = Axis2::default();
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, 2.0, 3.0, 0.5).unwrap();
    let aT: f64 = 1.0;

    let aD1: GeomCurveD1 = aCurve.eval_d1(aT).unwrap();
    let aP1: Point3 = aCurve.eval_d0(aT + precision::confusion()).unwrap();
    let aP2: Point3 = aCurve.eval_d0(aT - precision::confusion()).unwrap();
    let aFD = Vector3::from((aP1.xyz() - aP2.xyz()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD1.d1.x(), aFD.x(), THE_FD_TOL);
    EXPECT_NEAR(aD1.d1.y(), aFD.y(), THE_FD_TOL);
    EXPECT_NEAR(aD1.d1.z(), aFD.z(), THE_FD_TOL);
}

#[test]
fn GeomEval_SineWaveCurveTest__EvalD2_Analytical() {
    let anAx2 = Axis2::default();
    let (aA, aOm, aPhi) = (2.0, 3.0, 0.5);
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, aA, aOm, aPhi).unwrap();
    let aT: f64 = 1.0;

    let aD2: GeomCurveD2 = aCurve.eval_d2(aT).unwrap();

    // D2 = -A*omega^2*sin(omega*t+phi)*YDir = (0, -A*om^2*sin(...), 0)
    let aExpY: f64 = -aA * aOm * aOm * (aOm * aT + aPhi).sin();
    EXPECT_NEAR(aD2.d2.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aD2.d2.y(), aExpY, precision::confusion());
    EXPECT_NEAR(aD2.d2.z(), 0.0, precision::confusion());
}

#[test]
fn GeomEval_SineWaveCurveTest__ComparisonWithLine_ZeroAmplitude_Like() {
    // When amplitude approaches zero, D0 approaches a line.
    // We test D2 and D3 are near zero for very small amplitude.
    let anAx2 = Axis2::default();
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, 1e-10, 1.0, 0.0).unwrap();
    let aT: f64 = 1.0;

    let aD3: GeomCurveD3 = aCurve.eval_d3(aT).unwrap();
    EXPECT_NEAR(aD3.d2.magnitude(), 0.0, 1e-5);
    EXPECT_NEAR(aD3.d3.magnitude(), 0.0, 1e-5);
}

#[test]
fn GeomEval_SineWaveCurveTest__Transform_NotImplemented() {
    let anAx2 = Axis2::default();
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, 2.0, 3.0, 0.5).unwrap();
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));

    EXPECT_THROW(aCurve.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.transformed(aTrsf), "Standard_NotImplemented");
}

#[test]
fn GeomEval_SineWaveCurveTest__Copy_Independent() {
    let anAx2 = Axis2::default();
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, 2.0, 3.0, 0.5).unwrap();
    let aCopy = aCurve.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyCurve: Handle<GeomEvalSineWaveCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());
    EXPECT_NEAR(aCopyCurve.amplitude(), 2.0, precision::confusion());
    EXPECT_NEAR(aCopyCurve.omega(), 3.0, precision::confusion());
    EXPECT_NEAR(aCopyCurve.phase(), 0.5, precision::confusion());
}

#[test]
fn GeomEval_SineWaveCurveTest__DumpJson_NoCrash() {
    let anAx2 = Axis2::default();
    let aCurve = GeomEvalSineWaveCurve::new(anAx2, 2.0, 3.0, 0.0).unwrap();
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}
