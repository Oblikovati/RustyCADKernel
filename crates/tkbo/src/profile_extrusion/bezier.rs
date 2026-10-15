// SPDX-License-Identifier: GPL-2.0-only
//! Bézier pieces of each curve for the general loop region proof (ADR 0066). Arcs and
//! circles split into proper rational quadratic pieces of at most a quarter turn; authored
//! vertices are the exact piece ends, and each split point is computed once and shared.
use super::admitted::{AdmittedProfile, CircleCarrier};
use super::loop_geometry::{UseGeometry, use_geometry};
use super::loops::LoopUse;
use super::plane_math::on_circle;
use std::f64::consts::FRAC_PI_2;

/// One Bernstein piece in plane coordinates, traversed from its first to its last control.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BezierPiece {
    pub controls: Vec<[f64; 2]>,
    pub weights: Option<Vec<f64>>,
}

/// Pieces of one directed use, in traversal order.
pub(crate) fn use_pieces(profile: &AdmittedProfile, used: LoopUse) -> Vec<BezierPiece> {
    match use_geometry(profile, used) {
        UseGeometry::Segment { start, end } => vec![BezierPiece {
            controls: vec![start, end],
            weights: None,
        }],
        UseGeometry::Arc {
            carrier,
            start_angle,
            sweep,
            start,
            end,
        } => arc_pieces(carrier, [start_angle, sweep], [start, end]),
    }
}

/// Unit axis direction at a multiple of a quarter turn, if `angle` is one.
fn quarter_axis(angle: f64) -> Option<[f64; 2]> {
    let turns = (angle / FRAC_PI_2).round();
    if (angle - turns * FRAC_PI_2).abs() > 1e-12 {
        return None;
    }
    let axes = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
    Some(axes[(turns as i64).rem_euclid(4) as usize])
}

/// Plane point at `angle` on the carrier; quarter-turn points are placed exactly on the
/// axis directions so that quarter pieces can use the dyadic form below.
fn circle_point(carrier: CircleCarrier, angle: f64) -> [f64; 2] {
    match quarter_axis(angle) {
        Some(axis) => place(carrier, axis),
        None => on_circle(carrier.center, carrier.radius, angle),
    }
}

fn place(carrier: CircleCarrier, [x, y]: [f64; 2]) -> [f64; 2] {
    let [cx, cy] = carrier.center;
    [cx + carrier.radius * x, cy + carrier.radius * y]
}

/// Pieces of the arc that starts at `start_angle` and sweeps `sweep` between `start` and
/// `end` (the exact authored vertices).
fn arc_pieces(
    carrier: CircleCarrier,
    [start_angle, sweep]: [f64; 2],
    [start, end]: [[f64; 2]; 2],
) -> Vec<BezierPiece> {
    let count = (sweep.abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let step = sweep / count as f64;
    let angle = |index: usize| start_angle + step * index as f64;
    let mut ends: Vec<[f64; 2]> = (0..=count)
        .map(|i| circle_point(carrier, angle(i)))
        .collect();
    (ends[0], ends[count]) = (start, end);
    let piece = |index: usize| {
        let ends = [ends[index], ends[index + 1]];
        let middle = angle(index) + 0.5 * step;
        quarter_piece(carrier, [angle(index), angle(index + 1)], ends)
            .unwrap_or_else(|| general_piece(carrier, middle, 0.5 * step.abs(), ends))
    };
    (0..count).map(piece).collect()
}

/// An exact axis-aligned quarter: the stereographic parametrization
/// `((1 - t^2), 2t) / (1 + t^2)` turned by quarter turns, corner on the diagonal, weights
/// `(1, 1, 2)`. Its small dyadic weights keep the exact census and area arithmetic short;
/// a rational quadratic's curve depends only on `w1^2 / (w0 w2)`, so either traversal
/// sense of the quarter is the same circle arc.
fn quarter_piece(
    carrier: CircleCarrier,
    angles: [f64; 2],
    ends: [[f64; 2]; 2],
) -> Option<BezierPiece> {
    let [first, last] = [quarter_axis(angles[0])?, quarter_axis(angles[1])?];
    let quarter = (angles[1] - angles[0]).abs() <= FRAC_PI_2 + 1e-12;
    if !quarter || ends != [place(carrier, first), place(carrier, last)] {
        return None;
    }
    let corner = place(carrier, [first[0] + last[0], first[1] + last[1]]);
    Some(BezierPiece {
        controls: vec![ends[0], corner, ends[1]],
        weights: Some(vec![1.0, 1.0, 2.0]),
    })
}

/// A general proper arc piece of half-angle `half`: corner at the tangent intersection,
/// weights `(1, cos half, 1)`.
fn general_piece(
    carrier: CircleCarrier,
    middle: f64,
    half: f64,
    ends: [[f64; 2]; 2],
) -> BezierPiece {
    let corner = on_circle(carrier.center, carrier.radius / half.cos(), middle);
    BezierPiece {
        controls: vec![ends[0], corner, ends[1]],
        weights: Some(vec![1.0, half.cos(), 1.0]),
    }
}
