//! Direct translation of GeomEval_TBezierCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
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
use rustycad_tkg3d::geom_eval::GeomEvalTBezierCurve;
use rustycad_tkmath::gp::{Axis2, Point3, Transform3, Vector3};
use std::f64::consts::PI;

const THE_FD_TOL_D1: f64 = 1e-5;
const THE_FD_TOL_D2: f64 = 1e-4;

// Helper to create a T-Bezier that represents a unit semicircle in XY plane.
// C(t) = P_0 + P_1*sin(t) + P_2*cos(t) = (cos(t), sin(t), 0), t in [0, Pi].
fn createSemicircle() -> GeomEvalTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 0.0, 0.0)]);
    GeomEvalTBezierCurve::new(&aPoles, 1.0).unwrap()
}

// Helper to create a T-Bezier that represents a semi-ellipse (a=3, b=2) in XY plane.
// C(t) = P_0 + P_1*sin(t) + P_2*cos(t) = (3*cos(t), 2*sin(t), 0), t in [0, Pi].
fn createSemiEllipse() -> GeomEvalTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0, 0.0), (0.0, 2.0, 0.0), (3.0, 0.0, 0.0)]);
    GeomEvalTBezierCurve::new(&aPoles, 1.0).unwrap()
}

// Helper to create a simple 3-pole (order 1) T-Bezier curve
fn createSimpleCurve() -> GeomEvalTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)]);
    GeomEvalTBezierCurve::new(&aPoles, 1.0).unwrap()
}

// Test construction with valid parameters
#[test]
fn GeomEval_TBezierCurveTest__Construction_ValidParams() {
    let aCurve = createSimpleCurve();
    assert_eq!(aCurve.nb_poles(), 3);
    assert_eq!(aCurve.order(), 1);
    EXPECT_NEAR(aCurve.alpha(), 1.0, precision::confusion());
    assert!(!aCurve.is_rational());
}

// Test construction throws for invalid parameters
#[test]
fn GeomEval_TBezierCurveTest__Construction_InvalidParams_Throws() {
    // Even number of poles (not odd)
    let aPoles2 = poles(&[(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)]);
    EXPECT_THROW(
        GeomEvalTBezierCurve::new(&aPoles2, 1.0),
        "Standard_ConstructionError",
    );

    // Invalid alpha
    let aPoles3 = poles(&[(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0)]);
    EXPECT_THROW(
        GeomEvalTBezierCurve::new(&aPoles3, 0.0),
        "Standard_ConstructionError",
    );
    EXPECT_THROW(
        GeomEvalTBezierCurve::new(&aPoles3, -1.0),
        "Standard_ConstructionError",
    );
}

// Test EvalD0 at endpoints matches StartPoint/EndPoint
#[test]
fn GeomEval_TBezierCurveTest__EvalD0_Endpoints() {
    let aCurve = createSimpleCurve();

    let aStart: Point3 = aCurve.eval_d0(aCurve.first_parameter()).unwrap();
    let aEnd: Point3 = aCurve.eval_d0(aCurve.last_parameter()).unwrap();

    let aStartPoint: Point3 = aCurve.start_point().unwrap();
    let aEndPoint: Point3 = aCurve.end_point().unwrap();
    EXPECT_NEAR(aStart.distance(&aStartPoint), 0.0, precision::confusion());
    EXPECT_NEAR(aEnd.distance(&aEndPoint), 0.0, precision::confusion());
}

// Test parameter range
#[test]
fn GeomEval_TBezierCurveTest__ParameterRange() {
    let aCurve = createSimpleCurve();
    EXPECT_NEAR(aCurve.first_parameter(), 0.0, precision::confusion());
    EXPECT_NEAR(aCurve.last_parameter(), PI / 1.0, precision::confusion()); // Pi/alpha
}

// Test derivative consistency: compare EvalD1 vs finite difference of EvalD0
#[test]
fn GeomEval_TBezierCurveTest__EvalD1_ConsistentWithD0() {
    let aCurve = createSimpleCurve();
    let aU: f64 = PI / 3.0;

    let aD1: GeomCurveD1 = aCurve.eval_d1(aU).unwrap();
    let aP1: Point3 = aCurve.eval_d0(aU + precision::confusion()).unwrap();
    let aP2: Point3 = aCurve.eval_d0(aU - precision::confusion()).unwrap();

    let aFD = Vector3::from((aP1.xyz() - aP2.xyz()) / (2.0 * precision::confusion()));

    expect_near_vec(aD1.d1, aFD, THE_FD_TOL_D1);
}

// Test periodicity and closure
#[test]
fn GeomEval_TBezierCurveTest__PeriodicityAndClosure() {
    let aCurve = createSimpleCurve();
    assert!(!aCurve.is_periodic());
}

#[test]
fn GeomEval_TBezierCurveTest__Reverse_NotImplemented() {
    let aCurve = createSimpleCurve();
    EXPECT_THROW(aCurve.reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.reversed_parameter(0.5), "Standard_NotImplemented");
}

// Test rational construction
#[test]
fn GeomEval_TBezierCurveTest__Construction_Rational() {
    let aPoles = poles(&[(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)]);
    let aWeights = weights(&[1.0, 2.0, 1.0]);
    let aCurve = GeomEvalTBezierCurve::with_weights(&aPoles, &aWeights, 1.0).unwrap();
    assert!(aCurve.is_rational());
    assert_eq!(aCurve.weights().len(), 3);
}

// Test Transform/Transformed are not implemented.
#[test]
fn GeomEval_TBezierCurveTest__Transform_NotImplemented() {
    let aCurve = createSimpleCurve();
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));

    EXPECT_THROW(aCurve.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aCurve.transformed(aTrsf), "Standard_NotImplemented");
}

// Test Copy produces independent identical object
#[test]
fn GeomEval_TBezierCurveTest__Copy_Independent() {
    let aCurve = createSimpleCurve();
    let aCopy = aCurve.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyCurve: Handle<GeomEvalTBezierCurve> = aCopy.cast();
    assert!(!aCopyCurve.is_null());
    assert_eq!(aCopyCurve.nb_poles(), 3);
    EXPECT_NEAR(aCopyCurve.alpha(), 1.0, precision::confusion());
}

// Test DumpJson does not crash
#[test]
fn GeomEval_TBezierCurveTest__DumpJson_NoCrash() {
    let aCurve = createSimpleCurve();
    let mut aSS = DumpStream::default();
    aCurve.dump_json(&mut aSS, -1).unwrap();
}

// Test EvalD2 derivative D2 matches finite differences of D1
#[test]
fn GeomEval_TBezierCurveTest__EvalD2_ConsistentWithD1() {
    let aCurve = createSimpleCurve();
    let aU: f64 = PI / 4.0;

    let aD2: GeomCurveD2 = aCurve.eval_d2(aU).unwrap();
    let aD1Plus: GeomCurveD1 = aCurve.eval_d1(aU + precision::confusion()).unwrap();
    let aD1Minus: GeomCurveD1 = aCurve.eval_d1(aU - precision::confusion()).unwrap();

    let aFD =
        Vector3::from((*aD1Plus.d1.xyz() - *aD1Minus.d1.xyz()) / (2.0 * precision::confusion()));

    expect_near_vec(aD2.d2, aFD, THE_FD_TOL_D2);
}

// Test EvalD3 derivative D3 matches finite differences of D2
#[test]
fn GeomEval_TBezierCurveTest__EvalD3_ConsistentWithD2() {
    let aCurve = createSimpleCurve();
    let aU: f64 = PI / 3.0;

    let aD3: GeomCurveD3 = aCurve.eval_d3(aU).unwrap();
    let aD2Plus: GeomCurveD2 = aCurve.eval_d2(aU + precision::confusion()).unwrap();
    let aD2Minus: GeomCurveD2 = aCurve.eval_d2(aU - precision::confusion()).unwrap();

    let aFD =
        Vector3::from((*aD2Plus.d2.xyz() - *aD2Minus.d2.xyz()) / (2.0 * precision::confusion()));

    expect_near_vec(aD3.d3, aFD, THE_FD_TOL_D2);
}

// Test EvalDN(u,1) matches D1 from EvalD1
#[test]
fn GeomEval_TBezierCurveTest__EvalDN_ConsistentWithD1() {
    let aCurve = createSimpleCurve();
    let aU: f64 = PI / 6.0;

    let aD1: GeomCurveD1 = aCurve.eval_d1(aU).unwrap();
    let aDN: Vector3 = aCurve.eval_dn(aU, 1).unwrap();

    expect_near_vec(aD1.d1, aDN, precision::confusion());
}

// Test T-Bezier semicircle matches Geom_Circle D0
#[test]
fn GeomEval_TBezierCurveTest__EvalD0_MatchesCircle() {
    let aTBez = createSemicircle();
    let aCircle: Handle<GeomCircle> = Handle::new(GeomCircle::new(Axis2::default(), 1.0).unwrap());

    let aParams = [
        0.0,
        PI / 6.0,
        PI / 4.0,
        PI / 3.0,
        PI / 2.0,
        2.0 * PI / 3.0,
        3.0 * PI / 4.0,
        PI,
    ];
    for aU in aParams {
        let aPT: Point3 = aTBez.eval_d0(aU).unwrap();
        let aPC: Point3 = aCircle.eval_d0(aU).unwrap();
        EXPECT_NEAR(aPT.distance(&aPC), 0.0, precision::angular());
    }
}

// Test T-Bezier semicircle matches Geom_Circle D1/D2/D3
#[test]
fn GeomEval_TBezierCurveTest__EvalD3_MatchesCircle() {
    let aTBez = createSemicircle();
    let aCircle: Handle<GeomCircle> = Handle::new(GeomCircle::new(Axis2::default(), 1.0).unwrap());

    let aParams = [PI / 6.0, PI / 3.0, PI / 2.0, 2.0 * PI / 3.0];
    for aU in aParams {
        let aDT: GeomCurveD3 = aTBez.eval_d3(aU).unwrap();
        let aDC: GeomCurveD3 = aCircle.eval_d3(aU).unwrap();

        EXPECT_NEAR(aDT.point.distance(&aDC.point), 0.0, precision::angular());
        expect_near_vec(aDT.d1, aDC.d1, precision::angular());
        expect_near_vec(aDT.d2, aDC.d2, precision::angular());
        expect_near_vec(aDT.d3, aDC.d3, precision::angular());
    }
}

// Test T-Bezier semi-ellipse matches Geom_Ellipse D0/D1/D2
#[test]
fn GeomEval_TBezierCurveTest__EvalD2_MatchesEllipse() {
    let aTBez = createSemiEllipse();
    let anEllipse: Handle<GeomEllipse> =
        Handle::new(GeomEllipse::new(Axis2::default(), 3.0, 2.0).unwrap());

    let aParams = [PI / 6.0, PI / 4.0, PI / 2.0, 3.0 * PI / 4.0];
    for aU in aParams {
        let aDT: GeomCurveD2 = aTBez.eval_d2(aU).unwrap();
        let aDC: GeomCurveD2 = anEllipse.eval_d2(aU).unwrap();

        EXPECT_NEAR(aDT.point.distance(&aDC.point), 0.0, precision::angular());
        expect_near_vec(aDT.d1, aDC.d1, precision::angular());
        expect_near_vec(aDT.d2, aDC.d2, precision::angular());
    }
}

// Test EvalD0 at multiple evenly-spaced points
#[test]
fn GeomEval_TBezierCurveTest__EvalD0_MultiplePoints() {
    let aCurve = createSimpleCurve();
    let aFirst: f64 = aCurve.first_parameter();
    let aLast: f64 = aCurve.last_parameter();
    let aNbSamples: i32 = 10;
    let aStep: f64 = (aLast - aFirst) / f64::from(aNbSamples);

    let mut aPrev: Point3 = aCurve.eval_d0(aFirst).unwrap();
    for i in 1..=aNbSamples {
        let aU: f64 = aFirst + aStep * f64::from(i);
        let aPnt: Point3 = aCurve.eval_d0(aU).unwrap();

        // Verify the point has finite coordinates
        assert!(aPnt.x().is_finite());
        assert!(aPnt.y().is_finite());
        assert!(aPnt.z().is_finite());

        // Verify consecutive points don't jump too far (smoothness check)
        let aDist: f64 = aPrev.distance(&aPnt);
        assert!(aDist < 10.0 * aStep);

        aPrev = aPnt;
    }
}
