//! Direct translation of GeomBndLib_Curve2d_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! The suites are split across `geom_bnd_lib_curve2d/suites_N.rs` in the upstream order.
#![allow(non_snake_case)]
mod bnd_support;
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use bnd_support::{array1, curve2d, dir2d, get2, pnt2d};
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkgeombase::geom_bnd_lib::{Curve2d, sampling};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2d, Axis22d, Point2};
use std::f64::consts::PI;

// Helper: compare two Bnd_Box2d results within tolerance.
fn CompareBoxes2d(theNew: &BoundingBox2, theOld: &BoundingBox2, theTol: f64) {
    for (aN, aO) in get2(theNew).iter().zip(get2(theOld)) {
        EXPECT_NEAR(*aN, aO, theTol);
    }
}

// Helper: verify that the new box conservatively encloses the old box.
fn ExpectConservativeEnvelope2d(theNew: &BoundingBox2, theOld: &BoundingBox2, theTol: f64) {
    let [aXminN, aXmaxN, aYminN, aYmaxN] = get2(theNew);
    let [aXminO, aXmaxO, aYminO, aYmaxO] = get2(theOld);
    assert!(aXminN <= aXminO + theTol);
    assert!(aXmaxN >= aXmaxO - theTol);
    assert!(aYminN <= aYminO + theTol);
    assert!(aYmaxN >= aYmaxO - theTol);
}

fn adaptor(theCurve: &Handle<dyn Geom2dCurve>) -> Geom2dAdaptorCurve {
    Geom2dAdaptorCurve::with_curve(theCurve).unwrap()
}

// `new Geom2d_Circle(gp_Ax2d(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0)), theRadius)`.
fn circle(theRadius: f64) -> Handle<dyn Geom2dCurve> {
    let anAxis = Axis2d::new(pnt2d(0.0, 0.0), dir2d(1.0, 0.0));
    curve2d(Geom2dCircle::from_axis(anAxis, theRadius, true).unwrap())
}

// `gp_Ax22d(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0), gp_Dir2d(0.0, 1.0))`.
fn frame() -> Axis22d {
    Axis22d::with_directions(pnt2d(0.0, 0.0), dir2d(1.0, 0.0), dir2d(0.0, 1.0)).unwrap()
}

fn new_box(theCurve: &Handle<dyn Geom2dCurve>, theTol: f64) -> BoundingBox2 {
    let mut aBox = BoundingBox2::default();
    Curve2d::from_curve(theCurve)
        .unwrap()
        .add(theTol, &mut aBox)
        .unwrap();
    aBox
}

fn new_box_range(
    theCurve: &Handle<dyn Geom2dCurve>,
    theU1: f64,
    theU2: f64,
    theTol: f64,
) -> BoundingBox2 {
    let mut aBox = BoundingBox2::default();
    Curve2d::from_curve(theCurve)
        .unwrap()
        .add_range(theU1, theU2, theTol, &mut aBox)
        .unwrap();
    aBox
}

fn old_box(theAdaptor: &dyn Adaptor2dCurve2d, theTol: f64) -> BoundingBox2 {
    let mut aBox = BoundingBox2::default();
    Add2dCurve::add_adaptor(theAdaptor, theTol, &mut aBox).unwrap();
    aBox
}

fn old_box_range(
    theCurve: &Handle<dyn Geom2dCurve>,
    theU1: f64,
    theU2: f64,
    theTol: f64,
) -> BoundingBox2 {
    let mut aBox = BoundingBox2::default();
    Add2dCurve::add_adaptor_range(&adaptor(theCurve), theU1, theU2, theTol, &mut aBox).unwrap();
    aBox
}

include!("geom_bnd_lib_curve2d/suites_0.rs");
include!("geom_bnd_lib_curve2d/suites_1.rs");
