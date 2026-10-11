//! Native angle thresholds and transformation multiplication order are shared by camera rotations.
use rustycad_tkernel::{ncollection::Vec2, precision, standard::Failure};
use rustycad_tkmath::gp::{self, Axis1, Direction3, Point3, Transform3};
use std::f64::consts::{FRAC_PI_2, PI, TAU};
pub(super) fn rotation_pitch_delta(start: Direction3, delta: f64) -> f64 {
    let pitch = (-start.z()).asin();
    let limit = FRAC_PI_2 - PI / 180.0;
    (pitch + delta).clamp(-limit, limit) - pitch
}
pub(super) fn retained_rotation_transform(
    pivot: Point3,
    up: Direction3,
    direction: Direction3,
    angles: Vec2<f64>,
) -> Result<Option<Transform3>, Failure> {
    let has_yaw = angles.x.abs() > precision::angular();
    let has_pitch = angles.y.abs() > precision::angular();
    if !has_yaw && !has_pitch {
        return Ok(None);
    }
    let mut transform = Transform3::default();
    if has_yaw {
        multiply_rotation_axis(&mut transform, pivot, gp::dz(), angles.x)?;
    }
    if has_pitch {
        multiply_rotation_axis(&mut transform, pivot, up.crossed(direction)?, angles.y)?;
    }
    Ok(Some(transform))
}
pub(super) fn multiply_rotation_axis(
    transform: &mut Transform3,
    pivot: Point3,
    direction: Direction3,
    angle: f64,
) -> Result<(), Failure> {
    let mut rotation = Transform3::default();
    rotation.set_rotation(Axis1::new(pivot, direction), angle)?;
    transform.multiply(rotation);
    Ok(())
}
pub(super) fn wrap_orbit_angle(mut angle: f64) -> f64 {
    // Preserve the native repeated-subtraction rounding, including exact +/-2pi endpoints.
    while angle > TAU {
        angle -= TAU;
    }
    while angle < -TAU {
        angle += TAU;
    }
    angle
}
