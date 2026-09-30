//! Direct translation of Geom2dEval_CircleInvoluteCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::{
    Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry,
};
use rustycad_tkg2d::geom2d_eval::Geom2dEvalCircleInvoluteCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2d, Point2, Transform2, Vector2};
use std::f64::consts::PI;

const THE_FD_TOL: f64 = 1e-5;
const THE_FD_TOL_DN: f64 = 1e-4;

fn involute(radius: f64) -> Geom2dEvalCircleInvoluteCurve {
    Geom2dEvalCircleInvoluteCurve::new(Axis2d::default(), radius).unwrap()
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__Construction_ValidParams() {
    let aCurve = involute(1.0);
    EXPECT_NEAR(aCurve.radius(), 1.0, precision::confusion());
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__Construction_InvalidRadius_Throws() {
    let anAx2d = Axis2d::default();
    EXPECT_THROW(
        Geom2dEvalCircleInvoluteCurve::new(anAx2d, 0.0),
        "Standard_ConstructionError",
    );
    EXPECT_THROW(
        Geom2dEvalCircleInvoluteCurve::new(anAx2d, -1.0),
        "Standard_ConstructionError",
    );
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__EvalD0_AtT0_OnBaseCircle() {
    let aR: f64 = 1.0;
    let aCurve = involute(aR);

    // C(0) = R*(cos(0)+0, sin(0)-0) = (R, 0)
    let aP0: Point2 = aCurve.eval_d0(0.0).unwrap();
    EXPECT_NEAR(aP0.x(), aR, precision::confusion());
    EXPECT_NEAR(aP0.y(), 0.0, precision::confusion());
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__D1_AtT0_IsZero_Cusp() {
    let aCurve = involute(1.0);

    // D1(0) = R*0*(cos(0), sin(0)) = (0, 0)
    let aD1: Geom2dCurveD1 = aCurve.eval_d1(0.0).unwrap();
    EXPECT_NEAR(aD1.d1.magnitude(), 0.0, precision::confusion());
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__D1_SpeedLinearInParameter() {
    let aR: f64 = 2.0;
    let aCurve = involute(aR);

    // |D1(t)| = R*t
    let aParams: [f64; 5] = [0.5, 1.0, 2.0, PI, 5.0];
    for aT in aParams {
        let aD1: Geom2dCurveD1 = aCurve.eval_d1(aT).unwrap();
        EXPECT_NEAR(aD1.d1.magnitude(), aR * aT, precision::confusion());
    }
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__EvalD1_ConsistentWithD0() {
    let aCurve = involute(1.0);
    let aT: f64 = 2.0;

    let aD1: Geom2dCurveD1 = aCurve.eval_d1(aT).unwrap();
    let aP1: Point2 = aCurve.eval_d0(aT + precision::confusion()).unwrap();
    let aP2: Point2 = aCurve.eval_d0(aT - precision::confusion()).unwrap();
    let aFD = Vector2::from((aP1.xy() - aP2.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD1.d1.x(), aFD.x(), THE_FD_TOL);
    EXPECT_NEAR(aD1.d1.y(), aFD.y(), THE_FD_TOL);
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__EvalD2_ConsistentWithD1() {
    let aCurve = involute(1.0);
    let aT: f64 = 2.0;

    let aD2: Geom2dCurveD2 = aCurve.eval_d2(aT).unwrap();
    let aD1p: Geom2dCurveD1 = aCurve.eval_d1(aT + precision::confusion()).unwrap();
    let aD1m: Geom2dCurveD1 = aCurve.eval_d1(aT - precision::confusion()).unwrap();
    let aFD = Vector2::from((aD1p.d1.xy() - aD1m.d1.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD2.d2.x(), aFD.x(), THE_FD_TOL);
    EXPECT_NEAR(aD2.d2.y(), aFD.y(), THE_FD_TOL);
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__EvalD3_ConsistentWithD2() {
    let aCurve = involute(1.0);
    let aT: f64 = 2.0;

    let aD3: Geom2dCurveD3 = aCurve.eval_d3(aT).unwrap();
    let aD2p: Geom2dCurveD2 = aCurve.eval_d2(aT + precision::confusion()).unwrap();
    let aD2m: Geom2dCurveD2 = aCurve.eval_d2(aT - precision::confusion()).unwrap();
    let aFD = Vector2::from((aD2p.d2.xy() - aD2m.d2.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aD3.d3.x(), aFD.x(), THE_FD_TOL);
    EXPECT_NEAR(aD3.d3.y(), aFD.y(), THE_FD_TOL);
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__EvalDN_HigherOrder_ConsistentWithPreviousOrder() {
    let aCurve = involute(1.0);
    let aT: f64 = 2.0;
    let aN: i32 = 6;

    let aDN: Vector2 = aCurve.eval_dn(aT, aN).unwrap();
    let aDNm1P: Vector2 = aCurve.eval_dn(aT + precision::confusion(), aN - 1).unwrap();
    let aDNm1M: Vector2 = aCurve.eval_dn(aT - precision::confusion(), aN - 1).unwrap();
    let aFD = Vector2::from((aDNm1P.xy() - aDNm1M.xy()) / (2.0 * precision::confusion()));

    EXPECT_NEAR(aDN.x(), aFD.x(), THE_FD_TOL_DN);
    EXPECT_NEAR(aDN.y(), aFD.y(), THE_FD_TOL_DN);
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__Properties() {
    let aCurve = involute(1.0);
    assert!(!aCurve.is_closed().unwrap());
    assert!(!aCurve.is_periodic());
    assert_eq!(aCurve.continuity(), Shape::CN);
    EXPECT_NEAR(aCurve.first_parameter(), 0.0, precision::confusion());
    assert!(precision::is_infinite(aCurve.last_parameter()));
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__Reverse_NotImplemented() {
    let aCurve = involute(1.0);
    EXPECT_THROW(aCurve.reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.reversed_parameter(0.5), "Standard_NotImplemented");
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__Transform_NotImplemented() {
    let aCurve = involute(1.0);
    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(1.0, 2.0));
    EXPECT_THROW(aCurve.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.transformed(aTrsf), "Standard_NotImplemented");
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__Copy_Independent() {
    let aCurve = involute(3.0);
    let aCopy = aCurve.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyCurve: Handle<Geom2dEvalCircleInvoluteCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());
    EXPECT_NEAR(aCopyCurve.radius(), 3.0, precision::confusion());
}

#[test]
fn Geom2dEval_CircleInvoluteCurveTest__DumpJson_NoCrash() {
    let aCurve = involute(1.0);
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}
