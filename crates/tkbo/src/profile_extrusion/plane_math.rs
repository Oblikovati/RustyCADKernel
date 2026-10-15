// SPDX-License-Identifier: GPL-2.0-only
//! Small fixed-size vector helpers for admission and hypothesis construction. Topology
//! decisions never rest on these floating values alone; the certified proof checks them.
use super::profile::ArcSense;
use std::f64::consts::TAU;

pub(crate) fn sub2(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

pub(crate) fn length2(a: [f64; 2]) -> f64 {
    a[0].hypot(a[1])
}

pub(crate) fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(crate) fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(crate) fn norm3(a: [f64; 3]) -> f64 {
    dot3(a, a).sqrt()
}

/// Unit vector, or `None` for a zero or nonfinite input.
pub(crate) fn unit3(a: [f64; 3]) -> Option<[f64; 3]> {
    let norm = norm3(a);
    (norm.is_finite() && norm > 0.0).then(|| a.map(|c| c / norm))
}

/// Polar angle of `point` around `center`.
pub(crate) fn polar_angle(center: [f64; 2], point: [f64; 2]) -> f64 {
    let [dx, dy] = sub2(point, center);
    dy.atan2(dx)
}

/// Signed sweep from `start` to `end` angle in the arc sense: positive counterclockwise,
/// magnitude in `(0, 2pi]`.
pub(crate) fn signed_sweep(start: f64, end: f64, sense: ArcSense) -> f64 {
    let positive = |delta: f64| {
        let sweep = delta.rem_euclid(TAU);
        if sweep == 0.0 { TAU } else { sweep }
    };
    match sense {
        ArcSense::CounterClockwise => positive(end - start),
        ArcSense::Clockwise => -positive(start - end),
    }
}

/// Point of a circle at angle `angle`.
pub(crate) fn on_circle(center: [f64; 2], radius: f64, angle: f64) -> [f64; 2] {
    [
        center[0] + radius * angle.cos(),
        center[1] + radius * angle.sin(),
    ]
}
