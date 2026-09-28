// Copyright (c) 2026 OPEN CASCADE SAS
// This library is free software under LGPL version 2.1 with OCCT exception.
// See licenses/LICENSE_LGPL_21.txt and licenses/OCCT_LGPL_EXCEPTION.txt.
//! Poly_ArrayOfNodes_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkmath::{
    gp::{Point2, Point3},
    poly::{ArrayOfNodes, ArrayOfUvNodes},
};
#[test]
fn Poly_ArrayOfNodesTest__AssignBetweenPrecisions() {
    check_nodes_to_double();
    check_nodes_to_float();
}
fn check_nodes_to_double() {
    let mut floats = [[1.0_f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
    let source = ArrayOfNodes::from_float_slice(&mut floats).unwrap();
    let mut target = ArrayOfNodes::new(2).unwrap();
    target.assign(&source).unwrap();
    assert!(
        target
            .value(0)
            .unwrap()
            .is_equal(&Point3::new(1.0, 2.0, 3.0), 0.0)
    );
    assert!(
        target
            .value(1)
            .unwrap()
            .is_equal(&Point3::new(4.0, 5.0, 6.0), 0.0)
    );
}
fn check_nodes_to_float() {
    let mut doubles = [[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]];
    let source = ArrayOfNodes::from_double_slice(&mut doubles).unwrap();
    let mut target = ArrayOfNodes::default();
    target.set_double_precision(false).unwrap();
    target.resize(2, false).unwrap();
    target.assign(&source).unwrap();
    assert!(
        target
            .value(0)
            .unwrap()
            .is_equal(&Point3::new(7.0, 8.0, 9.0), 0.0)
    );
    assert!(
        target
            .value(1)
            .unwrap()
            .is_equal(&Point3::new(10.0, 11.0, 12.0), 0.0)
    );
}
#[test]
fn Poly_ArrayOfUVNodesTest__AssignBetweenPrecisions() {
    check_uv_to_double();
    check_uv_to_float();
}
fn check_uv_to_double() {
    let mut floats = [[1.0_f32, 2.0], [3.0, 4.0]];
    let source = ArrayOfUvNodes::from_float_slice(&mut floats).unwrap();
    let mut target = ArrayOfUvNodes::new(2).unwrap();
    target.assign(&source).unwrap();
    assert!(
        target
            .value(0)
            .unwrap()
            .is_equal(Point2::new(1.0, 2.0), 0.0)
    );
    assert!(
        target
            .value(1)
            .unwrap()
            .is_equal(Point2::new(3.0, 4.0), 0.0)
    );
}
fn check_uv_to_float() {
    let mut doubles = [[5.0, 6.0], [7.0, 8.0]];
    let source = ArrayOfUvNodes::from_double_slice(&mut doubles).unwrap();
    let mut target = ArrayOfUvNodes::default();
    target.set_double_precision(false).unwrap();
    target.resize(2, false).unwrap();
    target.assign(&source).unwrap();
    assert!(
        target
            .value(0)
            .unwrap()
            .is_equal(Point2::new(5.0, 6.0), 0.0)
    );
    assert!(
        target
            .value(1)
            .unwrap()
            .is_equal(Point2::new(7.0, 8.0), 0.0)
    );
}
