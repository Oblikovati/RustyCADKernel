// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Bnd_OBB behavior beyond the legacy suite: native quirks and the errors that replace
//! undefined native reads. The numeric results of the same paths are compared with OCCT
//! bit for bit by the `obb_build` and `obb_pair` probes.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::bnd::{BoundingBox3, OrientedBoundingBox};
use rustycad_tkmath::gp::{Direction3, Point3, Xyz};

const AXES: [Direction3; 3] = [Direction3::X, Direction3::Y, Direction3::Z];

fn points(values: &[[f64; 3]]) -> Array1<'static, Point3> {
    let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (index, [x, y, z]) in values.iter().enumerate() {
        array[index as i32 + 1] = Point3::new(*x, *y, *z);
    }
    array
}

#[test]
fn default_box_is_void_with_null_axes_and_is_out_everywhere() {
    let obb = OrientedBoundingBox::default();
    assert!(obb.is_void() && !obb.is_aa_box());
    assert_eq!(obb.x_direction(), Xyz::default());
    assert!(obb.vertices().is_none());
    // IsOut(gp_Pnt) has no void guard: |0| > -1 on every axis.
    assert!(obb.is_out_point(Point3::new(0.0, 0.0, 0.0)));
    let failure = obb.position().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}

#[test]
fn negative_sizes_are_kept_because_the_assertions_are_debug_only() {
    let mut obb = OrientedBoundingBox::new(Point3::new(0.0, 0.0, 0.0), AXES, 1.0, -2.0, 3.0);
    assert!(obb.is_void());
    obb.set_y_component(Direction3::Y, 2.0);
    assert!(!obb.is_void());
    obb.set_z_component(Direction3::Z, -0.5);
    assert_eq!(obb.z_h_size(), -0.5);
    // Enlarge adds the absolute gap even to a void box.
    obb.enlarge(-1.0);
    assert_eq!(
        [obb.x_h_size(), obb.y_h_size(), obb.z_h_size()],
        [2.0, 3.0, 0.5]
    );
    obb.set_void();
    assert_eq!(obb, OrientedBoundingBox::default());
}

#[test]
fn vertices_follow_the_native_bit_order_and_square_extent() {
    let mut obb = OrientedBoundingBox::new(Point3::new(1.0, 2.0, 3.0), AXES, 1.0, 2.0, 3.0);
    let vertices = obb.vertices().unwrap();
    assert_eq!(vertices[0], Point3::new(0.0, 0.0, 0.0));
    assert_eq!(vertices[1], Point3::new(2.0, 0.0, 0.0));
    assert_eq!(vertices[6], Point3::new(0.0, 4.0, 6.0));
    assert_eq!(obb.square_extent(), 4.0 * 14.0);
    obb.set_center(Point3::new(0.0, 0.0, 0.0));
    obb.set_x_component(Direction3::Y, 1.0);
    assert_eq!(obb.x_direction(), Xyz::new(0.0, 1.0, 0.0));
    let frame = obb.position().unwrap();
    assert_eq!(frame.direction(), Direction3::Z);
}

#[test]
fn from_box_uses_gap_adjusted_limits_and_void_boxes_stay_void() {
    let mut bounds = BoundingBox3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 4.0, 6.0));
    bounds.set_gap(1.0);
    let obb = OrientedBoundingBox::from_box(&bounds);
    assert!(obb.is_aa_box());
    assert_eq!(obb.center(), Xyz::new(1.0, 2.0, 3.0));
    assert_eq!(
        [obb.x_h_size(), obb.y_h_size(), obb.z_h_size()],
        [2.0, 3.0, 4.0]
    );
    assert_eq!(
        OrientedBoundingBox::from_box(&BoundingBox3::default()),
        OrientedBoundingBox::default()
    );
}

#[test]
fn one_and_two_point_rebuilds_use_the_closed_forms() {
    let mut obb = OrientedBoundingBox::default();
    let mut tolerances = Array1::with_bounds(1, 1).unwrap();
    tolerances[1] = -0.25;
    obb.rebuild(&points(&[[1.0, 2.0, 3.0]]), Some(&tolerances), false)
        .unwrap();
    assert!(obb.is_aa_box());
    assert_eq!(obb.x_h_size(), 0.25);
    // Two points closer than Confusion: one point enlarged by max(tol) + Confusion.
    obb.rebuild(&points(&[[0.0; 3], [5e-8, 0.0, 0.0]]), None, false)
        .unwrap();
    assert!(obb.is_aa_box());
    assert_eq!(obb.x_h_size(), precision::confusion());
    // A true segment: X along the segment, not axis aligned, extents cover both ends.
    obb.rebuild(&points(&[[0.0; 3], [0.0, 0.0, 4.0]]), None, true)
        .unwrap();
    assert!(!obb.is_aa_box());
    assert_eq!(obb.x_direction(), Xyz::new(0.0, 0.0, 1.0));
    assert_eq!(obb.y_direction(), Xyz::new(0.0, -1.0, 0.0));
    assert_eq!(
        (obb.x_h_size(), obb.center()),
        (2.0, Xyz::new(0.0, 0.0, 2.0))
    );
}

#[test]
fn many_point_rebuilds_cover_every_point_in_both_modes() {
    let cloud = points(&[
        [0.0, 0.0, 0.0],
        [4.0, 1.0, 0.0],
        [1.0, 3.0, 0.5],
        [2.0, 2.0, 5.0],
        [-1.0, 0.5, 2.0],
        [3.0, -2.0, 1.0],
    ]);
    for optimal in [false, true] {
        let mut obb = OrientedBoundingBox::default();
        obb.rebuild(&cloud, None, optimal).unwrap();
        let mut grown = obb;
        grown.enlarge(1e-9);
        assert!(cloud.iter().all(|point| grown.contains(*point)));
        assert!(grown.is_completely_inside(&obb));
    }
}

#[test]
fn empty_lists_and_mismatched_tolerance_bounds_raise_out_of_range() {
    let mut obb = OrientedBoundingBox::new(Point3::new(0.0, 0.0, 0.0), AXES, 1.0, 1.0, 1.0);
    let empty = Array1::<Point3>::default();
    let failure = obb.rebuild(&empty, None, false).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    let cloud = points(&[[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    let shifted = Array1::with_bounds(0, 3).unwrap();
    // DiTO fails in BuildBox after SetVoid; the optimal path fails while building its BVH.
    assert!(obb.rebuild(&cloud, Some(&shifted), false).is_err());
    assert!(obb.is_void());
    let before = OrientedBoundingBox::new(Point3::new(0.0, 0.0, 0.0), AXES, 1.0, 1.0, 1.0);
    obb = before;
    assert!(obb.rebuild(&cloud, Some(&shifted), true).is_err());
    assert_eq!(obb, before);
}

#[test]
fn add_copies_into_void_ignores_void_and_otherwise_rebuilds() {
    let other = OrientedBoundingBox::new(Point3::new(5.0, 5.0, 5.0), AXES, 1.0, 1.0, 1.0);
    let mut obb = OrientedBoundingBox::default();
    obb.add_box(&other).unwrap();
    assert_eq!(obb, other);
    obb.add_box(&OrientedBoundingBox::default()).unwrap();
    assert_eq!(obb, other);
    obb.add_point(Point3::new(9.0, 5.0, 5.0)).unwrap();
    assert!(obb.contains(Point3::new(9.0, 5.0, 5.0)));
    assert!(!obb.is_out_box(&other) && other.intersects(&obb));
}

#[test]
fn separating_axes_handle_rotated_boxes() {
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let rotated = [
        Direction3::new(s, s, 0.0).unwrap(),
        Direction3::new(-s, s, 0.0).unwrap(),
        Direction3::Z,
    ];
    let diamond = OrientedBoundingBox::new(Point3::new(0.0, 0.0, 0.0), rotated, 1.0, 1.0, 1.0);
    let near = OrientedBoundingBox::new(Point3::new(2.2, 0.0, 0.0), AXES, 1.0, 1.0, 1.0);
    let far = OrientedBoundingBox::new(Point3::new(2.5, 0.0, 0.0), AXES, 1.0, 1.0, 1.0);
    assert!(diamond.intersects(&near));
    assert!(diamond.is_out_box(&far) && far.is_out_box(&diamond));
    assert!(diamond.is_out_box(&OrientedBoundingBox::default()));
}

#[test]
fn dump_json_names_fields_like_the_native_macros() {
    let obb = OrientedBoundingBox::new(Point3::new(1.0, 2.0, 3.0), AXES, 0.5, 1.5, 2.5);
    let mut stream = DumpStream::default();
    obb.dump_json(&mut stream, 0).unwrap();
    assert_eq!(
        stream.text(),
        "\"className\": \"Bnd_OBB\", \"HDims[0]\": 0.5, \"HDims[1]\": 1.5, \"HDims[2]\": 2.5, \"IsAABox\": 0"
    );
    let mut nested = DumpStream::default();
    obb.dump_json(&mut nested, -1).unwrap();
    assert!(
        nested
            .text()
            .contains("\"Center\": {\"gp_XYZ\": [1, 2, 3]}, \"Axes[0]\": {\"gp_XYZ\": [1, 0, 0]}")
    );
}
