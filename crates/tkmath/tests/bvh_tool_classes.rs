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

//! The `BVH` namespace tool classes (`ToBndBox`, `VecComp`, `UnitVector`,
//! `MatrixOp`, `SurfaceCalculator`, `BVH_AxisSelector`) as public free functions.
//! Bit-exact agreement with the native build is checked by the `bvh_tools` probe.
use rustycad_tkernel::ncollection::{Mat4, Vec2, Vec3, Vec4};
use rustycad_tkmath::bvh::{
    BvhVector, main_axis, matrix_op_inverse, matrix_op_multiply, surface_area, to_bnd_box_scalar,
};

#[test]
fn scalar_to_bnd_box_spans_the_x_axis_only() {
    let widened = to_bnd_box_scalar(-1.5_f32, 2.25_f32);
    assert_eq!(widened.get().unwrap(), [-1.5, 0.0, 0.0, 2.25, 0.0, 0.0]);
    // Bnd_Box(gp_Pnt, gp_Pnt) stores the corners as given, even reversed.
    let reversed = to_bnd_box_scalar(3.0_f64, -3.0_f64);
    assert!(!reversed.is_void());
}

#[test]
fn surface_area_ignores_w_and_falls_back_for_degenerate_sizes() {
    assert_eq!(surface_area(&Vec3::new(1.0_f64, 2.0, 3.0)), 22.0);
    assert_eq!(surface_area(&Vec4::new(1.0_f64, 2.0, 3.0, 100.0)), 22.0);
    assert_eq!(surface_area(&Vec2::new(-2.0_f64, 3.0)), 6.0);
    // A flat 2D size has no area; the perimeter-like sum of extents replaces it.
    assert_eq!(surface_area(&Vec2::new(0.0_f64, -3.0)), 3.0);
    assert_eq!(surface_area(&Vec3::new(0.0_f32, 0.0, -4.0)), 4.0);
}

#[test]
fn matrix_op_inverts_and_projects_with_the_w_reciprocal() {
    let mut matrix = Mat4::<f64>::identity();
    matrix.set_diagonal(Vec4::new(2.0, 4.0, 1.0, 2.0));
    let mut inverse = Mat4::<f64>::identity();
    matrix_op_inverse(&matrix, &mut inverse);
    assert_eq!(inverse.get_diagonal(), Vec4::new(0.5, 0.25, 1.0, 0.5));
    let projected = matrix_op_multiply(&matrix, &Vec4::new(1.0, 1.0, 1.0, 1.0));
    assert_eq!(projected, Vec4::new(1.0, 2.0, 0.5, 1.0));
}

#[test]
fn axis_selector_uses_strict_comparisons_in_two_and_three_dimensions() {
    assert_eq!(main_axis(&Vec2::new(1.0_f64, 1.0)), 1);
    assert_eq!(main_axis(&Vec2::new(2.0_f64, 1.0)), 0);
    assert_eq!(main_axis(&Vec3::new(1.0_f64, 1.0, 1.0)), 0);
    assert_eq!(main_axis(&Vec3::new(1.0_f64, 2.0, 2.0)), 2);
    assert_eq!(main_axis(&Vec4::new(1.0_f64, 3.0, 2.0, 9.0)), 1);
}

#[test]
fn unit_vectors_and_components_follow_the_specializations() {
    assert_eq!(Vec2::<f32>::unit_z(), Vec2::new(0.0, 0.0));
    assert_eq!(Vec4::<f64>::unit_z(), Vec4::new(0.0, 0.0, 1.0, 0.0));
    // VecComp<T, 3>::Get maps every axis past Y to Z.
    assert_eq!(Vec3::new(1.0_f64, 2.0, 3.0).component(7), 3.0);
    assert_eq!(Vec4::new(1.0_f64, 2.0, 3.0, 4.0).component(3), 4.0);
}
