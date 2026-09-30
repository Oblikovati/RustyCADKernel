//! Translation of BRepBuilderAPI_Transform_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871, using the TKPrim box maker.
#![allow(non_snake_case)]
mod primitive_legacy_support;
use primitive_legacy_support::*;
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Transform3};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

#[test]
fn BRepBuilderAPI_TransformTest__Translate() {
    let shape = legacy_box_sizes(10.0, 10.0, 10.0);
    let result = legacy_transform(&shape, Transform3::translation([100.0, 0.0, 0.0]));
    let center = legacy_volume_properties(&result).centre_of_mass();
    assert!((center.x() - 105.0).abs() <= precision::confusion());
    assert!((center.y() - 5.0).abs() <= precision::confusion());
    assert!((center.z() - 5.0).abs() <= precision::confusion());
}

#[test]
fn BRepBuilderAPI_TransformTest__Rotate() {
    let shape = legacy_box_sizes(10.0, 10.0, 10.0);
    let axis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    let mut transform = Transform3::default();
    transform
        .set_rotation(axis, std::f64::consts::PI / 2.0)
        .unwrap();
    let result = legacy_transform(&shape, transform);
    let center = legacy_volume_properties(&result).centre_of_mass();
    assert!((center.x() + 5.0).abs() <= precision::confusion());
    assert!((center.y() - 5.0).abs() <= precision::confusion());
}

#[test]
fn BRepBuilderAPI_TransformTest__Scale() {
    let shape = legacy_box_sizes(10.0, 10.0, 10.0);
    let mut transform = Transform3::default();
    transform
        .set_scale(Point3::new(0.0, 0.0, 0.0), 2.0)
        .unwrap();
    let result = legacy_transform(&shape, transform);
    let volume = legacy_volume_properties(&result).mass();
    assert!((volume - 8000.0).abs() <= precision::confusion());
}

#[test]
fn BRepBuilderAPI_TransformTest__Mirror() {
    let shape = legacy_box_at(Point3::new(10.0, 0.0, 0.0));
    let normal = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let plane = Axis2::new(Point3::new(0.0, 0.0, 0.0), normal).unwrap();
    let mut transform = Transform3::default();
    transform.set_mirror_plane(plane);
    let result = legacy_transform(&shape, transform);
    assert!(legacy_volume_properties(&result).centre_of_mass().x() < 0.0);
}

#[test]
fn BRepBuilderAPI_TransformTest__ShapeValidity() {
    let shape = legacy_box_sizes(10.0, 10.0, 10.0);
    let result = legacy_transform(&shape, Transform3::translation([50.0, 50.0, 50.0]));
    let analyzer = BRepCheckAnalyzer::new(&result, true, false, false).unwrap();
    assert!(analyzer.is_valid().unwrap());
}
