//! Direct translation of GeomEval_CircularHelixCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
mod geom_eval_curve_common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use geom_eval_curve_common::expect_near_vec;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomGeometry,
};
use rustycad_tkg3d::geom_eval::GeomEvalCircularHelixCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Point3, Transform3, Vector3};
use std::f64::consts::PI;

const THE_FD_TOL: f64 = 1e-5;

fn helix(radius: f64, pitch: f64) -> GeomEvalCircularHelixCurve {
    GeomEvalCircularHelixCurve::new(Axis2::default(), radius, pitch).unwrap()
}

#[test]
fn GeomEval_CircularHelixCurveTest__Construction_ValidParams() {
    let aCurve = helix(5.0, 10.0);
    EXPECT_NEAR(aCurve.radius(), 5.0, precision::confusion());
    EXPECT_NEAR(aCurve.pitch(), 10.0, precision::confusion());
}

#[test]
fn GeomEval_CircularHelixCurveTest__Construction_InvalidRadius_Throws() {
    let anAx2 = Axis2::default();
    EXPECT_THROW(
        GeomEvalCircularHelixCurve::new(anAx2, 0.0, 10.0),
        "Standard_ConstructionError",
    );
    EXPECT_THROW(
        GeomEvalCircularHelixCurve::new(anAx2, -1.0, 10.0),
        "Standard_ConstructionError",
    );
}

#[test]
fn GeomEval_CircularHelixCurveTest__Construction_NegativePitch_NoThrow() {
    let anAx2 = Axis2::default();
    assert!(GeomEvalCircularHelixCurve::new(anAx2, 5.0, -10.0).is_ok());
}

#[test]
fn GeomEval_CircularHelixCurveTest__EvalD0_AtKnownPoints() {
    let (aR, aP) = (5.0, 10.0);
    let aCurve = helix(aR, aP);

    // t=0: C(0) = (R, 0, 0)
    let aP0: Point3 = aCurve.eval_d0(0.0).unwrap();
    EXPECT_NEAR(aP0.x(), aR, precision::confusion());
    EXPECT_NEAR(aP0.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP0.z(), 0.0, precision::confusion());

    // t=Pi/2: C = (0, R, P/4)
    let aP1: Point3 = aCurve.eval_d0(PI / 2.0).unwrap();
    EXPECT_NEAR(aP1.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP1.y(), aR, precision::confusion());
    EXPECT_NEAR(aP1.z(), aP / 4.0, precision::confusion());

    // t=Pi: C = (-R, 0, P/2)
    let aP2: Point3 = aCurve.eval_d0(PI).unwrap();
    EXPECT_NEAR(aP2.x(), -aR, precision::confusion());
    EXPECT_NEAR(aP2.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP2.z(), aP / 2.0, precision::confusion());

    // t=2*Pi: C = (R, 0, P) -- one full turn
    let aP3: Point3 = aCurve.eval_d0(2.0 * PI).unwrap();
    EXPECT_NEAR(aP3.x(), aR, precision::confusion());
    EXPECT_NEAR(aP3.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP3.z(), aP, precision::confusion());
}

#[test]
fn GeomEval_CircularHelixCurveTest__EvalD1_ConstantSpeed() {
    let (aR, aP) = (5.0, 10.0);
    let aCurve = helix(aR, aP);

    // |D1| should be constant = sqrt(R^2 + (P/(2*Pi))^2)
    let aZRate: f64 = aP / (2.0 * PI);
    let aExpSpeed: f64 = (aR * aR + aZRate * aZRate).sqrt();

    let aParams = [0.0, 0.5, 1.0, PI, 2.0 * PI];
    for aT in aParams {
        let aD1: GeomCurveD1 = aCurve.eval_d1(aT).unwrap();
        EXPECT_NEAR(aD1.d1.magnitude(), aExpSpeed, precision::confusion());
    }
}

#[test]
fn GeomEval_CircularHelixCurveTest__EvalD1_ConsistentWithD0() {
    let aCurve = helix(5.0, 10.0);
    let aT: f64 = 1.5;

    let aD1: GeomCurveD1 = aCurve.eval_d1(aT).unwrap();
    let aP1: Point3 = aCurve.eval_d0(aT + precision::confusion()).unwrap();
    let aP2: Point3 = aCurve.eval_d0(aT - precision::confusion()).unwrap();
    let aFD = Vector3::from((aP1.xyz() - aP2.xyz()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD1.d1.x(), aFD.x(), THE_FD_TOL);
    EXPECT_NEAR(aD1.d1.y(), aFD.y(), THE_FD_TOL);
    EXPECT_NEAR(aD1.d1.z(), aFD.z(), THE_FD_TOL);
}

#[test]
fn GeomEval_CircularHelixCurveTest__D1_D2_Orthogonal() {
    let aCurve = helix(5.0, 10.0);

    // D2 has no Z component, so D1.D2 = R^2*(-sin*(-cos) + cos*(-sin)) = 0
    let aParams = [0.0, 1.0, PI, 3.5];
    for aT in aParams {
        let aD2: GeomCurveD2 = aCurve.eval_d2(aT).unwrap();
        // D2 magnitude should be R
        EXPECT_NEAR(aD2.d2.magnitude(), 5.0, precision::confusion());
    }
}

#[test]
fn GeomEval_CircularHelixCurveTest__ComparisonWithCircle_ZeroPitch() {
    let anAx2 = Axis2::default();
    let aR: f64 = 5.0;
    let aHelix = GeomEvalCircularHelixCurve::new(anAx2, aR, 0.0).unwrap();
    let aCircle: Handle<GeomCircle> = Handle::new(GeomCircle::new(anAx2, aR).unwrap());

    let aParams = [0.0, PI / 4.0, PI / 2.0, PI, 3.0 * PI / 2.0];
    for aT in aParams {
        let aPH: Point3 = aHelix.eval_d0(aT).unwrap();
        let aPC: Point3 = aCircle.eval_d0(aT).unwrap();
        EXPECT_NEAR(aPH.x(), aPC.x(), precision::confusion());
        EXPECT_NEAR(aPH.y(), aPC.y(), precision::confusion());
        EXPECT_NEAR(aPH.z(), aPC.z(), precision::confusion());

        let aDH1: GeomCurveD1 = aHelix.eval_d1(aT).unwrap();
        let aDC1: GeomCurveD1 = aCircle.eval_d1(aT).unwrap();
        expect_near_vec(aDH1.d1, aDC1.d1, precision::angular());

        let aDH2: GeomCurveD2 = aHelix.eval_d2(aT).unwrap();
        let aDC2: GeomCurveD2 = aCircle.eval_d2(aT).unwrap();
        expect_near_vec(aDH2.d2, aDC2.d2, precision::angular());

        let aDH3: GeomCurveD3 = aHelix.eval_d3(aT).unwrap();
        let aDC3: GeomCurveD3 = aCircle.eval_d3(aT).unwrap();
        expect_near_vec(aDH3.d3, aDC3.d3, precision::angular());
    }
}

#[test]
fn GeomEval_CircularHelixCurveTest__EvalDN_CyclesPeriod4() {
    let aCurve = helix(5.0, 10.0);
    let aT: f64 = 1.0;

    // DN(t,4) XY-components should equal DN(t,0) XY = D0 XY contribution
    // More specifically: DN cycles with period 4 in XY
    let aD1: Vector3 = aCurve.eval_dn(aT, 1).unwrap();
    let aD5: Vector3 = aCurve.eval_dn(aT, 5).unwrap();
    EXPECT_NEAR(aD1.x(), aD5.x(), precision::confusion());
    EXPECT_NEAR(aD1.y(), aD5.y(), precision::confusion());
    // Z: D1 has Z component, D5 does not
    EXPECT_NEAR(aD5.z(), 0.0, precision::confusion());
}

#[test]
fn GeomEval_CircularHelixCurveTest__Transform_NotImplemented() {
    let aCurve = helix(5.0, 10.0);
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));

    EXPECT_THROW(aCurve.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.transformed(aTrsf), "Standard_NotImplemented");
}

#[test]
fn GeomEval_CircularHelixCurveTest__Copy_Independent() {
    let aCurve = helix(5.0, 10.0);
    let aCopy = aCurve.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyCurve: Handle<GeomEvalCircularHelixCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());
    EXPECT_NEAR(aCopyCurve.radius(), 5.0, precision::confusion());
    EXPECT_NEAR(aCopyCurve.pitch(), 10.0, precision::confusion());
}

#[test]
fn GeomEval_CircularHelixCurveTest__Reverse_NotImplemented() {
    let aCurve = helix(5.0, 10.0);
    EXPECT_THROW(aCurve.reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.reversed_parameter(3.0), "Standard_NotImplemented");
}

#[test]
fn GeomEval_CircularHelixCurveTest__Properties() {
    let aCurve = helix(5.0, 10.0);
    assert!(!aCurve.is_closed().unwrap());
    assert!(!aCurve.is_periodic());
    assert_eq!(aCurve.continuity(), Shape::CN);
    assert!(aCurve.is_cn(100).unwrap());
}

#[test]
fn GeomEval_CircularHelixCurveTest__DumpJson_NoCrash() {
    let aCurve = helix(5.0, 10.0);
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}
