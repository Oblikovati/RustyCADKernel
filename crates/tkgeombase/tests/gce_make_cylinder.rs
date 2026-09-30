//! Direct translation of gce_MakeCylinder_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gce::{ErrorType, MakeCylinder, Root};
use rustycad_tkmath::gp::{Axis1, Cylinder3, Direction3, Point3};

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

#[test]
fn gce_MakeCylinderTest__FromAxis_YBranch_DoneAndOrthogonalXAxis() {
    let aAxis = Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 1.0, 1.0));

    let aMaker = MakeCylinder::from_axis(aAxis, 5.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aCylinder: Cylinder3 = aMaker.value().unwrap();
    EXPECT_NEAR(aCylinder.radius(), 5.0, precision::confusion());
    assert!(
        aCylinder
            .axis()
            .direction()
            .is_parallel(aAxis.direction(), precision::angular())
    );

    let aDot: f64 = aCylinder
        .axis()
        .direction()
        .xyz()
        .dot(aCylinder.position().x_direction().xyz());
    EXPECT_NEAR(aDot, 0.0, precision::angular());
}

#[test]
fn gce_MakeCylinderTest__FromPoints_ZBranch_DoneAndOrthogonalXAxis() {
    let aP1 = Point3::new(0.0, 0.0, 0.0);
    let aP2 = Point3::new(0.0, 0.0, 10.0);
    let aP3 = Point3::new(3.0, 4.0, 0.0);

    let aMaker = MakeCylinder::through_points(aP1, aP2, aP3).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aCylinder: Cylinder3 = aMaker.value().unwrap();
    EXPECT_NEAR(aCylinder.radius(), 5.0, precision::confusion());
    let aAxis = Direction3::from_xyz(aP2.xyz() - aP1.xyz()).unwrap();
    assert!(
        aCylinder
            .axis()
            .direction()
            .is_parallel(aAxis, precision::angular())
    );

    let aDot: f64 = aCylinder
        .axis()
        .direction()
        .xyz()
        .dot(aCylinder.position().x_direction().xyz());
    EXPECT_NEAR(aDot, 0.0, precision::angular());
}

#[test]
fn gce_MakeCylinderTest__FromAxis_NegativeRadius_NegativeRadius() {
    let aAxis = Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));

    let aMaker = MakeCylinder::from_axis(aAxis, -1.0).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::NegativeRadius);
}
