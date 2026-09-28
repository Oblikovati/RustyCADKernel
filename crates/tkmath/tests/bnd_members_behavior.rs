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

//! Bnd_Box, Bnd_Box2d, Bnd_B2/Bnd_B3 members the legacy suites do not call, and
//! Bnd_Tools. The same members are compared with OCCT by the `box_extra`, `box2_extra`
//! and `compact_trsf` probes.
use rustycad_tkmath::bnd::{
    BoundingBox2, BoundingBox3, BoxLimits2, BoxLimits3, CompactBox2, CompactBox3, box2_to_bvh,
    box3_to_bvh,
};
use rustycad_tkmath::gp::{
    Axis1, Direction2, Direction3, Point2, Point3, Transform2, Transform3, Vector2, Xy, Xyz,
};

#[test]
fn box3_limits_getters_and_update_point() {
    let mut bounds = BoundingBox3::default();
    bounds.update_point(1.0, 2.0, 3.0);
    bounds.update_point(-1.0, 5.0, 3.0);
    bounds.set_gap(0.5);
    bounds.open_zmax();
    let expected = BoxLimits3 {
        x_min: -1.5,
        x_max: 1.5,
        y_min: 1.5,
        y_max: 5.5,
        z_min: 2.5,
        z_max: 1e100,
    };
    assert_eq!(bounds.limits(), expected);
    assert_eq!((bounds.x_min(), bounds.z_max()), (-1.5, 1e100));
    // The unchecked Get() of a void box reads the sentinels shifted by the gap.
    let mut void = BoundingBox3::default();
    void.set_gap(1.0);
    assert_eq!(void.limits().x_min, f64::MAX - 1.0);
}

#[test]
fn box_dumps_write_raw_sides_and_gap() {
    let mut bounds = BoundingBox3::new(Point3::new(0.0, -1.5, 2.0), Point3::new(1.0, 2.0, 1e-7));
    bounds.set_gap(0.25);
    bounds.open_ymax();
    let mut text = Vec::new();
    bounds.dump(&mut text).unwrap();
    assert_eq!(
        String::from_utf8(text).unwrap(),
        "Box3D : \n Xmin : 0\n Xmax : 1\n Ymin : -1.5\n Ymax : Infinite\n Zmin : 2\n Zmax : 1e-07\n Gap : 0.25\n"
    );
    let mut text = Vec::new();
    BoundingBox2::default().dump(&mut text).unwrap();
    assert_eq!(String::from_utf8(text).unwrap(), "Box2d : Void\n Gap : 0\n");
    let mut whole = BoundingBox3::default();
    whole.set_whole();
    let mut text = Vec::new();
    whole.dump(&mut text).unwrap();
    assert_eq!(
        String::from_utf8(text).unwrap(),
        "Box3D : Whole\n Gap : 0\n"
    );
}

#[test]
fn box2_rays_limits_and_transformed_queries() {
    let mut bounds = BoundingBox2::default();
    bounds.set_ray(Point2::new(1.0, 2.0), Direction2::new(-1.0, 0.0).unwrap());
    let expected = BoxLimits2 {
        x_min: -1e100,
        x_max: 1.0,
        y_min: 2.0,
        y_max: 2.0,
    };
    assert_eq!(bounds.limits(), expected);
    bounds.add_ray(Point2::new(3.0, 0.0), Direction2::new(0.0, 1.0).unwrap());
    bounds.update_point(4.0, -1.0);
    assert_eq!(
        (bounds.x_max(), bounds.y_min(), bounds.y_max()),
        (4.0, -1.0, 1e100)
    );
    let mut other = BoundingBox2::default();
    other.update_bounds([10.0, 10.0], [11.0, 11.0]);
    let mut shift = Transform2::default();
    shift.set_translation(Vector2::new(-20.0, 0.0));
    assert!(!bounds.is_out_transformed(&other, shift));
    assert!(
        bounds.is_out_both(Transform2::default(), &other, Transform2::default())
            == bounds.is_out_box(&other)
    );
}

#[test]
fn tools_convert_limits_and_reject_void_boxes() {
    let mut bounds = BoundingBox3::new(Point3::new(0.0, 1.0, 2.0), Point3::new(3.0, 4.0, 5.0));
    bounds.set_gap(1.0);
    let converted = box3_to_bvh(&bounds).unwrap();
    assert_eq!(
        (converted.corner_min().x, converted.corner_max().z),
        (-1.0, 6.0)
    );
    let failure = box3_to_bvh(&BoundingBox3::default()).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    let mut flat = BoundingBox2::default();
    flat.update_bounds([0.0, 1.0], [2.0, 3.0]);
    flat.open_xmin();
    let converted = box2_to_bvh(&flat).unwrap();
    assert_eq!(
        (converted.corner_min().x, converted.corner_max().y),
        (-1e100, 3.0)
    );
    assert!(box2_to_bvh(&BoundingBox2::default()).is_err());
}

#[test]
fn compact_boxes_test_transformed_separation_and_inclusion() {
    let inner = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));
    let outer = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(3.0, 3.0, 3.0));
    let mut rotation = Transform3::default();
    let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::Z);
    rotation.set_rotation(axis, 0.3).unwrap();
    assert!(inner.is_in_transformed_box(&outer, rotation));
    assert!(!outer.is_in_transformed_box(&inner, rotation));
    let mut far = Transform3::default();
    far.set_scale(Point3::new(10.0, 0.0, 0.0), -1.0).unwrap();
    assert!(inner.is_out_transformed_box(&inner, far));
    assert!(!inner.is_out_transformed_box(&outer, Transform3::default()));
    let flat_inner = CompactBox2::<f32>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));
    let flat_outer = CompactBox2::<f32>::new(Xy::new(0.0, 0.0), Xy::new(3.0, 3.0));
    let mut turn = Transform2::default();
    turn.set_rotation(Point2::new(0.0, 0.0), 0.3);
    assert!(flat_inner.is_in_transformed_box(&flat_outer, turn));
    assert!(!flat_inner.is_out_transformed_box(&flat_outer, turn));
}
