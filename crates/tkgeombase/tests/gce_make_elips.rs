//! Direct translation of gce_MakeElips_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gce::{ErrorType, MakeElips, Root};
use rustycad_tkmath::gp::{self, Ellipse3, Point3};

#[test]
fn gce_MakeElipsTest__FromPoints_Valid_Done() {
    let aCenter = Point3::new(0.0, 0.0, 0.0);
    let aS1 = Point3::new(10.0, 0.0, 0.0);
    let aS2 = Point3::new(0.0, 5.0, 0.0);

    let aMaker = MakeElips::from_points(aS1, aS2, aCenter).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let anElips: Ellipse3 = aMaker.value().unwrap();
    EXPECT_NEAR(anElips.major_radius(), 10.0, precision::confusion());
    EXPECT_NEAR(anElips.minor_radius(), 5.0, precision::confusion());
}

#[test]
fn gce_MakeElipsTest__FromPoints_Collinear_InvertAxis() {
    let aCenter = Point3::new(0.0, 0.0, 0.0);
    let aS1 = Point3::new(10.0, 0.0, 0.0);
    let aS2 = Point3::new(5.0, 0.0, 0.0);

    let aMaker = MakeElips::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::InvertAxis);
}

#[test]
fn gce_MakeElipsTest__FromPoints_TinyScale_NullAxis() {
    let aRes: f64 = gp::resolution();
    let aCenter = Point3::new(0.0, 0.0, 0.0);
    let aS1 = Point3::new(2.0 * aRes, 0.0, 0.0);
    let aS2 = Point3::new(0.0, 1.1 * aRes, 0.0);

    let aMaker = MakeElips::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::NullAxis);
}
