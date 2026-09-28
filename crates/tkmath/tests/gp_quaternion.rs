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

//! Complete OCC25574 legacy suite, including every Euler sequence and frame setup.
#![allow(non_snake_case)]
use rustycad_tkmath::gp::{
    Axis2, Direction3, EulerSequence as E, Matrix3, Point3, Quaternion, Transform3, Vector3, Xyz,
};
const NAMES: [&str; 24] = [
    "Extrinsic_XYZ",
    "Extrinsic_XZY",
    "Extrinsic_YZX",
    "Extrinsic_YXZ",
    "Extrinsic_ZXY",
    "Extrinsic_ZYX",
    "Intrinsic_XYZ",
    "Intrinsic_XZY",
    "Intrinsic_YZX",
    "Intrinsic_YXZ",
    "Intrinsic_ZXY",
    "Intrinsic_ZYX",
    "Extrinsic_XYX",
    "Extrinsic_XZX",
    "Extrinsic_YZY",
    "Extrinsic_YXY",
    "Extrinsic_ZYZ",
    "Extrinsic_ZXZ",
    "Intrinsic_XYX",
    "Intrinsic_XZX",
    "Intrinsic_YZY",
    "Intrinsic_YXY",
    "Intrinsic_ZXZ",
    "Intrinsic_ZYZ",
];
fn input_quaternion() -> Quaternion {
    let mut quaternion = Quaternion::default();
    quaternion.set(
        0.06766916507860499,
        0.21848101129786085,
        0.11994599260380681,
        0.9660744746954637,
    );
    quaternion
}
#[test]
fn gp_QuaternionTest__OCC25574_EulerAnglesConsistency() {
    let quaternion = input_quaternion();
    let inverse = quaternion.matrix().inverted().unwrap();
    let mut identity = Matrix3::default();
    identity.set_identity();
    for (index, order) in E::ALL.into_iter().enumerate() {
        let [alpha, beta, gamma] = quaternion.euler_angles(order);
        let mut second = Quaternion::default();
        second.set_euler_angles(order, alpha, beta, gamma);
        let difference = second.matrix() * inverse - identity;
        assert!(
            difference.determinant() <= 1e-5,
            "Round-trip failed for Euler sequence {}",
            NAMES[index]
        );
    }
}
#[test]
fn gp_QuaternionTest__OCC25574_EulerAxisRotationPreservesAxis() {
    for (index, order) in E::ALL.into_iter().enumerate() {
        for angle_index in 0..3 {
            assert_axis_rotation(order, NAMES[index], angle_index);
        }
    }
}
fn assert_axis_rotation(order: E, name: &str, angle_index: usize) {
    let axis = i32::from(name.as_bytes()[10 + angle_index]) - i32::from(b'X');
    assert!(axis >= 0);
    assert!(axis <= 2);
    let mut angles = [0.0; 3];
    angles[angle_index] = 0.5 * std::f64::consts::PI;
    let mut quaternion = Quaternion::default();
    quaternion.set_euler_angles(order, angles[0], angles[1], angles[2]);
    let mut vector = Xyz::default();
    vector.set_coord(axis + 1, 1.0).unwrap();
    let transformation = Transform3::from_quaternion(quaternion);
    let transformed = transformation.apply_coordinates(vector);
    assert!(
        (vector - transformed).square_modulus() <= 1e-7 * 1e-7,
        "Rotation around axis should not move a point on that axis. Sequence: {name}, angle index: {angle_index}"
    );
}
#[test]
fn gp_QuaternionTest__OCC25574_ExtrinsicIntrinsicCorrespondence() {
    let pairs = [
        (E::ExtrinsicXyz, E::IntrinsicZyx),
        (E::ExtrinsicXzy, E::IntrinsicYzx),
        (E::ExtrinsicYzx, E::IntrinsicXzy),
        (E::ExtrinsicYxz, E::IntrinsicZxy),
        (E::ExtrinsicZxy, E::IntrinsicYxz),
        (E::ExtrinsicZyx, E::IntrinsicXyz),
    ];
    let pair_names = [
        "XYZ/ZYX", "XZY/YZX", "YZX/XZY", "YXZ/ZXY", "ZXY/YXZ", "ZYX/XYZ",
    ];
    let mut quaternion = Quaternion::default();
    for (index, (first, second)) in pairs.into_iter().enumerate() {
        assert_pair(&mut quaternion, first, second, pair_names[index]);
    }
}
fn assert_pair(quaternion: &mut Quaternion, first: E, second: E, name: &str) {
    let (alpha, beta, gamma) = (0.1517461713131, 1.5162198410141, 1.9313156236541);
    quaternion.set_euler_angles(first, alpha, beta, gamma);
    let [gamma2, beta2, alpha2] = quaternion.euler_angles(second);
    assert!(
        (alpha - alpha2).abs() <= 1e-5,
        "alpha mismatch for pair {name}"
    );
    assert!(
        (beta - beta2).abs() <= 1e-5,
        "beta mismatch for pair {name}"
    );
    assert!(
        (gamma - gamma2).abs() <= 1e-5,
        "gamma mismatch for pair {name}"
    );
}
#[test]
fn gp_QuaternionTest__OCC25574_YawPitchRollRoundTrip() {
    let world = Axis2::default();
    let (alpha, beta, gamma) = (
        0.0,
        -35.0 / 180.0 * std::f64::consts::PI,
        90.0 / 180.0 * std::f64::consts::PI,
    );
    let result = yaw_pitch_roll_frame(world, alpha, beta, gamma);
    let transformation = Transform3::displacement(Axis2::default().axis3(), result.axis3());
    let [computed_alpha, computed_beta, computed_gamma] = transformation
        .rotation_quaternion()
        .euler_angles(E::YawPitchRoll);
    assert!(
        (alpha - computed_alpha).abs() <= 1e-5,
        "YawPitchRoll alpha mismatch"
    );
    assert!(
        (beta - computed_beta).abs() <= 1e-5,
        "YawPitchRoll beta mismatch"
    );
    assert!(
        (gamma - computed_gamma).abs() <= 1e-5,
        "YawPitchRoll gamma mismatch"
    );
}
fn yaw_pitch_roll_frame(world: Axis2, alpha: f64, beta: f64, gamma: f64) -> Axis2 {
    let rotation_z = Quaternion::from_axis_angle(Vector3::from(world.direction()), alpha).unwrap();
    let rotation_y = rotation_z.multiply_vector(Vector3::from(world.y_direction()));
    let rotation_x = rotation_z.multiply_vector(Vector3::from(world.x_direction()));
    let yaw = Quaternion::from_axis_angle(rotation_y, beta).unwrap();
    let rotation_z2 = yaw.multiply_vector(Vector3::from(world.direction()));
    let rotation_x2 = yaw.multiply_vector(rotation_x);
    let roll = Quaternion::from_axis_angle(rotation_x2, gamma).unwrap();
    let rotation_z3 = roll.multiply_vector(rotation_z2);
    Axis2::with_x_direction(
        Point3::default(),
        Direction3::from_vector(rotation_z3).unwrap(),
        Direction3::from_vector(rotation_x2).unwrap(),
    )
    .unwrap()
}
#[test]
fn gp_QuaternionTest__OCC25574_IntrinsicZYX_vs_ExtrinsicXYZ() {
    let quaternion = input_quaternion();
    let [alpha, beta, gamma] = quaternion.euler_angles(E::IntrinsicZyx);
    let [alpha2, beta2, gamma2] = quaternion.euler_angles(E::ExtrinsicXyz);
    assert!(
        (alpha - gamma2).abs() <= 1e-5,
        "Intrinsic ZYX alpha should equal Extrinsic XYZ gamma"
    );
    assert!((beta - beta2).abs() <= 1e-5, "beta should match");
    assert!(
        (gamma - alpha2).abs() <= 1e-5,
        "Intrinsic ZYX gamma should equal Extrinsic XYZ alpha"
    );
}
