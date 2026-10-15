// SPDX-License-Identifier: GPL-2.0-only
//! Interior seed hypothesis per loop: step inward from the midpoint of the loop's first
//! use to half the distance of the nearest boundary crossing. The certified loop proof
//! locates each seed exactly and must name this loop as the seed's outer loop.
use super::admitted::AdmittedProfile;
use super::loop_geometry::{UseGeometry, midpoint_frame, use_geometry};
use super::loops::{LoopUse, ProfileLoop};
use super::plane_math::*;

/// One seed per loop, inside the loop and outside all of its children.
/// `orientation` is the sign of each loop's area. `None` when no inward crossing exists.
pub(crate) fn loop_seeds(
    profile: &AdmittedProfile,
    loops: &[ProfileLoop],
    orientation: &[f64],
) -> Option<Vec<[f64; 2]>> {
    let boundary = all_uses(profile, loops);
    loops
        .iter()
        .zip(orientation)
        .map(|(closed, sign)| loop_seed(profile, closed, *sign, &boundary))
        .collect()
}

fn all_uses(profile: &AdmittedProfile, loops: &[ProfileLoop]) -> Vec<UseGeometry> {
    loops
        .iter()
        .flat_map(|closed| closed.uses.iter())
        .map(|used| use_geometry(profile, *used))
        .collect()
}

fn loop_seed(
    profile: &AdmittedProfile,
    closed: &ProfileLoop,
    sign: f64,
    boundary: &[UseGeometry],
) -> Option<[f64; 2]> {
    let first: LoopUse = closed.uses[0];
    let (origin, tangent) = midpoint_frame(use_geometry(profile, first));
    // Counterclockwise loops have their interior to the left of the traversal.
    let inward = [-sign * tangent[1], sign * tangent[0]];
    let floor = profile.resolution.weld;
    let hit = boundary
        .iter()
        .flat_map(|geometry| ray_hits(origin, inward, *geometry))
        .filter(|t| *t > floor)
        .fold(f64::INFINITY, f64::min);
    hit.is_finite().then(|| {
        [
            origin[0] + 0.5 * hit * inward[0],
            origin[1] + 0.5 * hit * inward[1],
        ]
    })
}

/// Ray parameters `t >= 0` where `origin + t * direction` meets the use.
fn ray_hits(origin: [f64; 2], direction: [f64; 2], geometry: UseGeometry) -> Vec<f64> {
    match geometry {
        UseGeometry::Segment { start, end } => segment_hit(origin, direction, start, end)
            .into_iter()
            .collect(),
        UseGeometry::Arc {
            carrier,
            start_angle,
            sweep,
            ..
        } => circle_hits(origin, direction, carrier.center, carrier.radius)
            .into_iter()
            .filter(|t| {
                let point = [origin[0] + t * direction[0], origin[1] + t * direction[1]];
                within_sweep(polar_angle(carrier.center, point), start_angle, sweep)
            })
            .collect(),
    }
}

fn segment_hit(
    origin: [f64; 2],
    direction: [f64; 2],
    start: [f64; 2],
    end: [f64; 2],
) -> Option<f64> {
    let edge = sub2(end, start);
    let denominator = direction[0] * edge[1] - direction[1] * edge[0];
    if denominator == 0.0 {
        return None;
    }
    let offset = sub2(start, origin);
    let t = (offset[0] * edge[1] - offset[1] * edge[0]) / denominator;
    let s = (offset[0] * direction[1] - offset[1] * direction[0]) / denominator;
    (t >= 0.0 && (0.0..=1.0).contains(&s)).then_some(t)
}

fn circle_hits(origin: [f64; 2], direction: [f64; 2], center: [f64; 2], radius: f64) -> Vec<f64> {
    let offset = sub2(origin, center);
    let b = offset[0] * direction[0] + offset[1] * direction[1];
    let c = offset[0] * offset[0] + offset[1] * offset[1] - radius * radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return Vec::new();
    }
    let root = discriminant.sqrt();
    [-b - root, -b + root]
        .into_iter()
        .filter(|t| *t >= 0.0)
        .collect()
}

/// Whether `angle` lies on the arc that starts at `start` and sweeps `sweep` radians.
fn within_sweep(angle: f64, start: f64, sweep: f64) -> bool {
    let along = (angle - start) * sweep.signum();
    along.rem_euclid(std::f64::consts::TAU) <= sweep.abs()
}
