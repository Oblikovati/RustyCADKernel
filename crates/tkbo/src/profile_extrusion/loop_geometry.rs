// SPDX-License-Identifier: GPL-2.0-only
//! Directed use geometry in plane coordinates, closed-form signed loop area and the
//! interior seed hypothesis of each loop. The certified loop proof re-decides every
//! topological fact derived here; these values only shape the request it checks.
use super::admitted::{AdmittedCurve, AdmittedProfile, CircleCarrier};
use super::loops::{LoopUse, ProfileLoop};
use super::plane_math::*;
use super::profile::ArcSense;

/// One directed use resolved to its traversal geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum UseGeometry {
    Segment {
        start: [f64; 2],
        end: [f64; 2],
    },
    /// `sweep` is signed in traversal direction (positive counterclockwise).
    Arc {
        carrier: CircleCarrier,
        start_angle: f64,
        sweep: f64,
        start: [f64; 2],
        end: [f64; 2],
    },
}

/// Traversal geometry of `used` within `profile`.
pub(crate) fn use_geometry(profile: &AdmittedProfile, used: LoopUse) -> UseGeometry {
    let forward = forward_geometry(profile, profile.curve(used.curve));
    if used.reversed {
        reverse(forward)
    } else {
        forward
    }
}

fn forward_geometry(profile: &AdmittedProfile, curve: &AdmittedCurve) -> UseGeometry {
    match *curve {
        AdmittedCurve::Line { ends, .. } => UseGeometry::Segment {
            start: profile.point(ends[0]),
            end: profile.point(ends[1]),
        },
        AdmittedCurve::Arc {
            ends,
            carrier,
            sense,
            ..
        } => arc_geometry(carrier, ends.map(|vertex| profile.point(vertex)), sense),
        AdmittedCurve::Circle { carrier, .. } => {
            // Coincident ends sweep one full counterclockwise turn from parameter 0.
            let start = on_circle(carrier.center, carrier.radius, 0.0);
            arc_geometry(carrier, [start, start], ArcSense::CounterClockwise)
        }
    }
}

fn arc_geometry(
    carrier: CircleCarrier,
    [start, end]: [[f64; 2]; 2],
    sense: ArcSense,
) -> UseGeometry {
    let start_angle = polar_angle(carrier.center, start);
    let sweep = signed_sweep(start_angle, polar_angle(carrier.center, end), sense);
    UseGeometry::Arc {
        carrier,
        start_angle,
        sweep,
        start,
        end,
    }
}

fn reverse(geometry: UseGeometry) -> UseGeometry {
    match geometry {
        UseGeometry::Segment { start, end } => UseGeometry::Segment {
            start: end,
            end: start,
        },
        UseGeometry::Arc {
            carrier,
            start_angle,
            sweep,
            start,
            end,
        } => UseGeometry::Arc {
            carrier,
            start_angle: start_angle + sweep,
            sweep: -sweep,
            start: end,
            end: start,
        },
    }
}

/// Green's-theorem contribution `1/2 * integral (x dy - y dx)` of one directed use.
fn green_term(geometry: UseGeometry) -> f64 {
    let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
    match geometry {
        UseGeometry::Segment { start, end } => 0.5 * cross(start, end),
        UseGeometry::Arc {
            carrier,
            sweep,
            start,
            end,
            ..
        } => {
            let chord = cross(carrier.center, sub2(end, start));
            0.5 * (chord + carrier.radius * carrier.radius * sweep)
        }
    }
}

/// Closed-form signed area of a loop (positive when counterclockwise) and its rounding
/// estimate: a small multiple of the unit roundoff times the summed term magnitudes.
pub(crate) fn signed_area(profile: &AdmittedProfile, closed: &ProfileLoop) -> (f64, f64) {
    let terms: Vec<f64> = closed
        .uses
        .iter()
        .map(|used| green_term(use_geometry(profile, *used)))
        .collect();
    let scale: f64 = terms.iter().map(|term| term.abs()).sum();
    (terms.iter().sum(), 64.0 * f64::EPSILON * scale)
}

/// Midpoint and unit tangent of a directed use, in traversal direction.
pub(crate) fn midpoint_frame(geometry: UseGeometry) -> ([f64; 2], [f64; 2]) {
    match geometry {
        UseGeometry::Segment { start, end } => {
            let delta = sub2(end, start);
            let length = length2(delta);
            let mid = [0.5 * (start[0] + end[0]), 0.5 * (start[1] + end[1])];
            (mid, [delta[0] / length, delta[1] / length])
        }
        UseGeometry::Arc {
            carrier,
            start_angle,
            sweep,
            ..
        } => {
            let angle = start_angle + 0.5 * sweep;
            let sign = sweep.signum();
            let tangent = [-sign * angle.sin(), sign * angle.cos()];
            (on_circle(carrier.center, carrier.radius, angle), tangent)
        }
    }
}
