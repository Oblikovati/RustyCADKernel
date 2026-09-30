//! Direct translation of GC_MakeCircle2d_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gc::{ErrorType, MakeCircle2d, Root};
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Point2};

fn base_circle() -> Circle2 {
    let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    Circle2::from_x_axis(axis, 5.0, true).unwrap()
}

#[test]
fn GC_MakeCircle2dTest__FromCircAndPoint_Done() {
    let aBase = base_circle();
    let aPoint = Point2::new(0.0, 10.0);

    let aMaker = MakeCircle2d::from_circ_point(aBase, aPoint).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    let aRadius: f64 = aMaker.value().unwrap().circ2d().unwrap().radius();
    EXPECT_NEAR(aRadius, 10.0, precision::confusion());
}

#[test]
fn GC_MakeCircle2dTest__FromCircAndDist_Done() {
    let aBase = base_circle();

    let aMaker = MakeCircle2d::from_circ_distance(aBase, -2.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    let aRadius: f64 = aMaker.value().unwrap().circ2d().unwrap().radius();
    EXPECT_NEAR(aRadius, 3.0, precision::confusion());
}

#[test]
fn GC_MakeCircle2dTest__FromCircAndPoint_AtCenter_DoneZeroRadius() {
    let aBase = base_circle();

    let aMaker = MakeCircle2d::from_circ_point(aBase, Point2::new(0.0, 0.0)).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    assert!(!aMaker.value().unwrap().is_null());
    let aRadius: f64 = aMaker.value().unwrap().circ2d().unwrap().radius();
    EXPECT_NEAR(aRadius, 0.0, precision::confusion());
}
