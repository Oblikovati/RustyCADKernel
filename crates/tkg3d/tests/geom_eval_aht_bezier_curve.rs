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

//! Direct translation of GeomEval_AHTBezierCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
mod geom_eval_curve_common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use geom_eval_curve_common::{expect_near_vec, poles, weights};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{
    GeomBoundedCurve, GeomCircle, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomEllipse,
    GeomGeometry,
};
use rustycad_tkg3d::geom_eval::GeomEvalAHTBezierCurve;
use rustycad_tkmath::gp::{Axis2, Point3, Transform3, Vector3};

const THE_FD_TOL_D1: f64 = 1e-5;
const THE_FD_TOL_D2: f64 = 1e-4;

// Helper to create a curve with basis {1, t, sinh(alpha*t), cosh(alpha*t), sin(beta*t),
// cos(beta*t)} algDegree=1, alpha=1.0, beta=1.0 => basisDim = 2 + 2 + 2 = 6 poles needed
fn createFullBasisCurve() -> GeomEvalAHTBezierCurve {
    let aPoles = poles(&[
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (0.0, 1.0, 0.0),
        (0.0, 1.0, 1.0),
        (1.0, 0.0, 1.0),
    ]);
    GeomEvalAHTBezierCurve::new(&aPoles, 1, 1.0, 1.0).unwrap()
}

// Helper to create a pure trigonometric AHT curve: basis {1, sin(t), cos(t)} => 3 poles
// C(t) = P_0 + P_1*sin(t) + P_2*cos(t) = (cos(t), sin(t), 0), t in [0, 1].
fn createTrigCircleArc() -> GeomEvalAHTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 0.0, 0.0)]);
    GeomEvalAHTBezierCurve::new(&aPoles, 0, 0.0, 1.0).unwrap()
}

// Helper to create a pure trigonometric AHT ellipse arc: basis {1, sin(t), cos(t)} => 3 poles
// C(t) = (3*cos(t), 2*sin(t), 0), t in [0, 1].
fn createTrigEllipseArc() -> GeomEvalAHTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0, 0.0), (0.0, 2.0, 0.0), (3.0, 0.0, 0.0)]);
    GeomEvalAHTBezierCurve::new(&aPoles, 0, 0.0, 1.0).unwrap()
}

// Helper to create a pure polynomial curve: basis {1, t, t^2} => 3 poles
fn createPolynomialCurve() -> GeomEvalAHTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)]);
    GeomEvalAHTBezierCurve::new(&aPoles, 2, 0.0, 0.0).unwrap()
}

// Test construction with full AHT basis
#[test]
fn GeomEval_AHTBezierCurveTest__Construction_FullBasis() {
    let aCurve = createFullBasisCurve();
    assert_eq!(aCurve.nb_poles(), 6);
    assert_eq!(aCurve.alg_degree(), 1);
    EXPECT_NEAR(aCurve.alpha(), 1.0, precision::confusion());
    EXPECT_NEAR(aCurve.beta(), 1.0, precision::confusion());
    assert!(!aCurve.is_rational());
}

// Test construction with pure polynomial basis
#[test]
fn GeomEval_AHTBezierCurveTest__Construction_Polynomial() {
    let aCurve = createPolynomialCurve();
    assert_eq!(aCurve.nb_poles(), 3);
    assert_eq!(aCurve.alg_degree(), 2);
    EXPECT_NEAR(aCurve.alpha(), 0.0, precision::confusion());
    EXPECT_NEAR(aCurve.beta(), 0.0, precision::confusion());
}

// Test parameter range is [0, 1]
#[test]
fn GeomEval_AHTBezierCurveTest__ParameterRange() {
    let aCurve = createFullBasisCurve();
    EXPECT_NEAR(aCurve.first_parameter(), 0.0, precision::confusion());
    EXPECT_NEAR(aCurve.last_parameter(), 1.0, precision::confusion());
}

// Test EvalD0 at endpoints matches StartPoint/EndPoint
#[test]
fn GeomEval_AHTBezierCurveTest__EvalD0_Endpoints() {
    let aCurve = createFullBasisCurve();

    let aStart: Point3 = aCurve.eval_d0(aCurve.first_parameter()).unwrap();
    let aEnd: Point3 = aCurve.eval_d0(aCurve.last_parameter()).unwrap();

    let aStartPoint: Point3 = aCurve.start_point().unwrap();
    let aEndPoint: Point3 = aCurve.end_point().unwrap();
    EXPECT_NEAR(aStart.distance(&aStartPoint), 0.0, precision::confusion());
    EXPECT_NEAR(aEnd.distance(&aEndPoint), 0.0, precision::confusion());
}

// Test derivative consistency: compare EvalD1 vs finite difference of EvalD0
#[test]
fn GeomEval_AHTBezierCurveTest__EvalD1_ConsistentWithD0() {
    let aCurve = createFullBasisCurve();
    let aU: f64 = 0.4;

    let aD1: GeomCurveD1 = aCurve.eval_d1(aU).unwrap();
    let aP1: Point3 = aCurve.eval_d0(aU + precision::confusion()).unwrap();
    let aP2: Point3 = aCurve.eval_d0(aU - precision::confusion()).unwrap();

    let aFD = Vector3::from((aP1.xyz() - aP2.xyz()) / (2.0 * precision::confusion()));

    expect_near_vec(aD1.d1, aFD, THE_FD_TOL_D1);
}

// Test periodicity
#[test]
fn GeomEval_AHTBezierCurveTest__Periodicity() {
    let aCurve = createFullBasisCurve();
    assert!(!aCurve.is_periodic());
}

#[test]
fn GeomEval_AHTBezierCurveTest__Reverse_NotImplemented() {
    let aCurve = createFullBasisCurve();
    EXPECT_THROW(aCurve.reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.reversed_parameter(0.5), "Standard_NotImplemented");
}

// Test rational construction
#[test]
fn GeomEval_AHTBezierCurveTest__Construction_Rational() {
    let aPoles = poles(&[(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)]);
    let aWeights = weights(&[1.0, 2.0, 1.0]);
    let aCurve = GeomEvalAHTBezierCurve::with_weights(&aPoles, &aWeights, 2, 0.0, 0.0).unwrap();
    assert!(aCurve.is_rational());
}

// Test Transform/Transformed are not implemented.
#[test]
fn GeomEval_AHTBezierCurveTest__Transform_NotImplemented() {
    let aCurve = createFullBasisCurve();
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));

    EXPECT_THROW(aCurve.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.transformed(aTrsf), "Standard_NotImplemented");
}

// Test Copy produces independent identical object
#[test]
fn GeomEval_AHTBezierCurveTest__Copy_Independent() {
    let aCurve = createFullBasisCurve();
    let aCopy = aCurve.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyCurve: Handle<GeomEvalAHTBezierCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());
    assert_eq!(aCopyCurve.nb_poles(), 6);
    assert_eq!(aCopyCurve.alg_degree(), 1);
}

// Test DumpJson does not crash
#[test]
fn GeomEval_AHTBezierCurveTest__DumpJson_NoCrash() {
    let aCurve = createFullBasisCurve();
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}

// Test EvalD2 derivative D2 matches finite differences of D1
#[test]
fn GeomEval_AHTBezierCurveTest__EvalD2_ConsistentWithD1() {
    let aCurve = createFullBasisCurve();
    let aU: f64 = 0.3;

    let aD2: GeomCurveD2 = aCurve.eval_d2(aU).unwrap();
    let aD1Plus: GeomCurveD1 = aCurve.eval_d1(aU + precision::confusion()).unwrap();
    let aD1Minus: GeomCurveD1 = aCurve.eval_d1(aU - precision::confusion()).unwrap();

    let aFD =
        Vector3::from((*aD1Plus.d1.xyz() - *aD1Minus.d1.xyz()) / (2.0 * precision::confusion()));

    expect_near_vec(aD2.d2, aFD, THE_FD_TOL_D2);
}

// Test EvalD3 derivative D3 matches finite differences of D2
#[test]
fn GeomEval_AHTBezierCurveTest__EvalD3_ConsistentWithD2() {
    let aCurve = createFullBasisCurve();
    let aU: f64 = 0.4;

    let aD3: GeomCurveD3 = aCurve.eval_d3(aU).unwrap();
    let aD2Plus: GeomCurveD2 = aCurve.eval_d2(aU + precision::confusion()).unwrap();
    let aD2Minus: GeomCurveD2 = aCurve.eval_d2(aU - precision::confusion()).unwrap();

    let aFD =
        Vector3::from((*aD2Plus.d2.xyz() - *aD2Minus.d2.xyz()) / (2.0 * precision::confusion()));

    expect_near_vec(aD3.d3, aFD, THE_FD_TOL_D2);
}

// Test EvalDN(u,1) matches D1 from EvalD1
#[test]
fn GeomEval_AHTBezierCurveTest__EvalDN_ConsistentWithD1() {
    let aCurve = createFullBasisCurve();
    let aU: f64 = 0.5;

    let aD1: GeomCurveD1 = aCurve.eval_d1(aU).unwrap();
    let aDN: Vector3 = aCurve.eval_dn(aU, 1).unwrap();

    expect_near_vec(aD1.d1, aDN, precision::confusion());
}

// Test EvalD0 for pure polynomial curve with known values
// Basis {1, t, t^2} on [0,1], P(t) = P0*1 + P1*t + P2*t^2
#[test]
fn GeomEval_AHTBezierCurveTest__EvalD0_PurePolynomial_KnownValues() {
    let aP0 = Point3::new(0.0, 0.0, 0.0);
    let aP1 = Point3::new(0.5, 1.0, 0.0);
    let aP2 = Point3::new(1.0, 0.0, 0.0);

    let aPoles = poles(&[(0.0, 0.0, 0.0), (0.5, 1.0, 0.0), (1.0, 0.0, 0.0)]);
    let aCurve = GeomEvalAHTBezierCurve::new(&aPoles, 2, 0.0, 0.0).unwrap();

    // P(0) = P0
    let aVal0: Point3 = aCurve.eval_d0(0.0).unwrap();
    EXPECT_NEAR(aVal0.distance(&aP0), 0.0, precision::confusion());

    // P(0.5) = P0 + 0.5*P1 + 0.25*P2
    let aExpected05 = Point3::new(
        aP0.x() + 0.5 * aP1.x() + 0.25 * aP2.x(),
        aP0.y() + 0.5 * aP1.y() + 0.25 * aP2.y(),
        aP0.z() + 0.5 * aP1.z() + 0.25 * aP2.z(),
    );
    let aVal05: Point3 = aCurve.eval_d0(0.5).unwrap();
    EXPECT_NEAR(aVal05.distance(&aExpected05), 0.0, precision::confusion());

    // P(1) = P0 + P1 + P2
    let aExpected1 = Point3::new(
        aP0.x() + aP1.x() + aP2.x(),
        aP0.y() + aP1.y() + aP2.y(),
        aP0.z() + aP1.z() + aP2.z(),
    );
    let aVal1: Point3 = aCurve.eval_d0(1.0).unwrap();
    EXPECT_NEAR(aVal1.distance(&aExpected1), 0.0, precision::confusion());
}

// Test AHT-Bezier trigonometric circle arc matches Geom_Circle D0
#[test]
fn GeomEval_AHTBezierCurveTest__EvalD0_MatchesCircle() {
    let aAHT = createTrigCircleArc();
    let aCircle: Handle<GeomCircle> = Handle::new(GeomCircle::new(Axis2::default(), 1.0).unwrap());

    let aParams = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
    for aU in aParams {
        let aPAHT: Point3 = aAHT.eval_d0(aU).unwrap();
        let aPCirc: Point3 = aCircle.eval_d0(aU).unwrap();
        EXPECT_NEAR(aPAHT.distance(&aPCirc), 0.0, precision::angular());
    }
}

// Test AHT-Bezier trigonometric circle arc matches Geom_Circle D1/D2/D3
#[test]
fn GeomEval_AHTBezierCurveTest__EvalD3_MatchesCircle() {
    let aAHT = createTrigCircleArc();
    let aCircle: Handle<GeomCircle> = Handle::new(GeomCircle::new(Axis2::default(), 1.0).unwrap());

    let aParams = [0.1, 0.25, 0.5, 0.75, 0.9];
    for aU in aParams {
        let aDT: GeomCurveD3 = aAHT.eval_d3(aU).unwrap();
        let aDC: GeomCurveD3 = aCircle.eval_d3(aU).unwrap();

        EXPECT_NEAR(aDT.point.distance(&aDC.point), 0.0, precision::angular());
        expect_near_vec(aDT.d1, aDC.d1, precision::angular());
        expect_near_vec(aDT.d2, aDC.d2, precision::angular());
        expect_near_vec(aDT.d3, aDC.d3, precision::angular());
    }
}

// Test AHT-Bezier trigonometric ellipse arc matches Geom_Ellipse D0/D1/D2
#[test]
fn GeomEval_AHTBezierCurveTest__EvalD2_MatchesEllipse() {
    let aAHT = createTrigEllipseArc();
    let anEllipse: Handle<GeomEllipse> =
        Handle::new(GeomEllipse::new(Axis2::default(), 3.0, 2.0).unwrap());

    let aParams = [0.1, 0.25, 0.5, 0.75, 0.9];
    for aU in aParams {
        let aDT: GeomCurveD2 = aAHT.eval_d2(aU).unwrap();
        let aDC: GeomCurveD2 = anEllipse.eval_d2(aU).unwrap();

        EXPECT_NEAR(aDT.point.distance(&aDC.point), 0.0, precision::angular());
        expect_near_vec(aDT.d1, aDC.d1, precision::angular());
        expect_near_vec(aDT.d2, aDC.d2, precision::angular());
    }
}
