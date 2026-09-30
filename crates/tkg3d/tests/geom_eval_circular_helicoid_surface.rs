//! Direct translation of GeomEval_CircularHelicoidSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
#[path = "geom_eval_surface_support/mod.rs"]
mod support;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{GeomCurve, GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2};
use rustycad_tkg3d::geom_eval::{GeomEvalCircularHelicoidSurface, GeomEvalCircularHelixCurve};
use rustycad_tkmath::gp::{Axis2, Axis3, Point3, Transform3, Vector3};
use std::f64::consts::PI;
use support::{EXPECT_VEC_NEAR, point_difference};

const THE_FD_TOL: f64 = 1e-5;

fn helicoid(pitch: f64) -> GeomEvalCircularHelicoidSurface {
    GeomEvalCircularHelicoidSurface::new(Axis3::default(), pitch).unwrap()
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__Construction_ValidParams() {
    EXPECT_NEAR(helicoid(10.0).pitch(), 10.0, precision::confusion());
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__Construction_ZeroPitch_Throws() {
    EXPECT_THROW(
        GeomEvalCircularHelicoidSurface::new(Axis3::default(), 0.0),
        "Standard_ConstructionError",
    );
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__Construction_NegativePitch_NoThrow() {
    assert!(GeomEvalCircularHelicoidSurface::new(Axis3::default(), -5.0).is_ok());
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__EvalD0_AtKnownPoints() {
    let aP: f64 = 10.0;
    let aSurf = helicoid(aP);

    let aP1: Point3 = aSurf.eval_d0(0.0, 1.0).unwrap();
    EXPECT_NEAR(aP1.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aP1.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP1.z(), 0.0, precision::confusion());

    let aP2: Point3 = aSurf.eval_d0(PI / 2.0, 1.0).unwrap();
    EXPECT_NEAR(aP2.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP2.y(), 1.0, precision::confusion());
    EXPECT_NEAR(aP2.z(), aP / 4.0, precision::confusion());

    let aP3: Point3 = aSurf.eval_d0(1.0, 0.0).unwrap();
    EXPECT_NEAR(aP3.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aP3.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aP3.z(), aP / (2.0 * PI), precision::confusion());
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__D2V_IsZero_RuledSurface() {
    let aSurf = helicoid(10.0);
    let aParams: [[f64; 2]; 3] = [[0.0, 1.0], [1.0, 2.0], [PI, -1.0]];
    for [aU, aV] in aParams {
        let aD2: GeomSurfaceD2 = aSurf.eval_d2(aU, aV).unwrap();
        EXPECT_NEAR(aD2.d2v.magnitude(), 0.0, precision::confusion());
    }
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__D1V_UnitRadial() {
    let aSurf = helicoid(10.0);
    let aParams: [f64; 5] = [0.0, 1.0, PI / 2.0, PI, 3.5];
    for aU in aParams {
        let aD1: GeomSurfaceD1 = aSurf.eval_d1(aU, 2.0).unwrap();
        EXPECT_NEAR(aD1.d1v.magnitude(), 1.0, precision::confusion());
    }
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__ComparisonWithHelix_ConstantV() {
    let anAx3 = Axis3::default();
    let anAx2 =
        Axis2::with_x_direction(anAx3.location(), anAx3.direction(), anAx3.x_direction()).unwrap();
    let (aP, aR) = (10.0, 3.0);
    let aSurf = GeomEvalCircularHelicoidSurface::new(anAx3, aP).unwrap();
    let aHelix = GeomEvalCircularHelixCurve::new(anAx2, aR, aP).unwrap();

    // S(u, R) should trace the same path as the helix C(u)
    let aParams: [f64; 5] = [0.0, 0.5, 1.0, PI, 2.0 * PI];
    for aU in aParams {
        let aPS: Point3 = aSurf.eval_d0(aU, aR).unwrap();
        let aPH: Point3 = aHelix.eval_d0(aU).unwrap();
        EXPECT_NEAR(aPS.distance(&aPH), 0.0, precision::confusion());
    }
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__EvalD1_ConsistentWithD0() {
    let aSurf = helicoid(10.0);
    let (aU, aV) = (1.0, 2.0);
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
fn GeomEval_CircularHelicoidSurfaceTest__Bounds_Infinite() {
    let (aU1, aU2, aV1, aV2) = helicoid(10.0).bounds();
    assert!(precision::is_infinite(aU1));
    assert!(precision::is_infinite(aU2));
    assert!(precision::is_infinite(aV1));
    assert!(precision::is_infinite(aV2));
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__Properties() {
    let aSurf = helicoid(10.0);
    assert!(!aSurf.is_u_closed().unwrap());
    assert!(!aSurf.is_v_closed().unwrap());
    assert!(!aSurf.is_u_periodic());
    assert!(!aSurf.is_v_periodic());
    EXPECT_THROW(aSurf.u_iso(0.0), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_iso(0.0), "Standard_NotImplemented");
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__Reverse_NotImplemented() {
    let aSurf = helicoid(10.0);
    EXPECT_THROW(aSurf.u_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reverse(), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.u_reversed_parameter(0.5), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.v_reversed_parameter(0.5), "Standard_NotImplemented");
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__Transform_NotImplemented() {
    let aSurf = helicoid(10.0);
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(1.0, 2.0, 3.0));
    EXPECT_THROW(aSurf.transform(aTrsf), "Standard_NotImplemented");
    EXPECT_THROW(aSurf.transformed(aTrsf), "Standard_NotImplemented");
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__Copy_Independent() {
    let aSurf = helicoid(10.0);
    let aCopy = aSurf.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopySurf = aCopy.cast::<GeomEvalCircularHelicoidSurface>();
    assert!(!aCopySurf.is_null());
    EXPECT_NEAR(aCopySurf.pitch(), 10.0, precision::confusion());
}

#[test]
fn GeomEval_CircularHelicoidSurfaceTest__DumpJson_NoCrash() {
    let aSurf = helicoid(10.0);
    let mut aSS = DumpStream::default();
    assert!(aSurf.dump_json(&mut aSS, -1).is_ok());
}
