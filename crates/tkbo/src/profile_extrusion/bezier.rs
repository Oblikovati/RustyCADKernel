// SPDX-License-Identifier: GPL-2.0-only
//! Bézier pieces of each curve for the plane graph region proof (ADR 0084). Circular and
//! elliptic arcs split into proper rational quadratic pieces of at most a quarter turn (an
//! elliptic piece is the affine image of a circular one, ADR 0083); splines split at their
//! knots. Authored vertices are the exact piece ends, and each split point is computed once
//! and shared.
use super::admitted::AdmittedProfile;
use super::loop_geometry::{Conic, UseGeometry, use_geometry};
use super::loops::LoopUse;
use super::plane_math::{length2, on_circle, sub2};
use std::f64::consts::FRAC_PI_2;

/// One Bernstein piece in plane coordinates, traversed from its first to its last control.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BezierPiece {
    pub controls: Vec<[f64; 2]>,
    pub weights: Option<Vec<f64>>,
}

impl BezierPiece {
    /// The same curve traversed backward.
    pub(crate) fn reversed(mut self) -> Self {
        self.controls.reverse();
        if let Some(weights) = self.weights.as_mut() {
            weights.reverse();
        }
        self
    }

    /// Largest control distance from the origin times the control polygon length: a bound
    /// scale for the Green integral's sensitivity to control rounding.
    pub(crate) fn green_scale(&self) -> f64 {
        let reach = self
            .controls
            .iter()
            .map(|p| length2(*p))
            .fold(0.0, f64::max);
        let polygon: f64 = self
            .controls
            .windows(2)
            .map(|w| length2(sub2(w[1], w[0])))
            .sum();
        reach * polygon
    }
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
        UseGeometry::Spline { pieces } => pieces,
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

/// Point at `angle` on the carrier; quarter-turn points are placed exactly on the axis
/// images so that quarter pieces can use the dyadic form below.
fn conic_point(carrier: Conic, angle: f64) -> [f64; 2] {
    match quarter_axis(angle) {
        Some(axis) => carrier.image(axis),
        None => carrier.point(angle),
    }
}

/// Pieces of the arc that starts at `start_angle` and sweeps `sweep` between `start` and
/// `end` (the exact authored vertices).
fn arc_pieces(
    carrier: Conic,
    [start_angle, sweep]: [f64; 2],
    [start, end]: [[f64; 2]; 2],
) -> Vec<BezierPiece> {
    let count = (sweep.abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let step = sweep / count as f64;
    let angle = |index: usize| start_angle + step * index as f64;
    let mut ends: Vec<[f64; 2]> = (0..=count)
        .map(|i| conic_point(carrier, angle(i)))
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
/// sense of the quarter is the same arc. An affine map keeps the form, so elliptic
/// quarters between axis ends use it too.
fn quarter_piece(carrier: Conic, angles: [f64; 2], ends: [[f64; 2]; 2]) -> Option<BezierPiece> {
    let [first, last] = [quarter_axis(angles[0])?, quarter_axis(angles[1])?];
    let quarter = (angles[1] - angles[0]).abs() <= FRAC_PI_2 + 1e-12;
    if !quarter || ends != [carrier.image(first), carrier.image(last)] {
        return None;
    }
    let corner = carrier.image([first[0] + last[0], first[1] + last[1]]);
    Some(BezierPiece {
        controls: vec![ends[0], corner, ends[1]],
        weights: Some(vec![1.0, 1.0, 2.0]),
    })
}

/// A general proper arc piece of half-angle `half`: corner at the tangent intersection (the
/// image of the circle's), weights `(1, cos half, 1)`.
fn general_piece(carrier: Conic, middle: f64, half: f64, ends: [[f64; 2]; 2]) -> BezierPiece {
    let corner = match carrier {
        Conic::Circle(c) => on_circle(c.center, c.radius / half.cos(), middle),
        Conic::Ellipse(_) => carrier.image([middle.cos() / half.cos(), middle.sin() / half.cos()]),
    };
    BezierPiece {
        controls: vec![ends[0], corner, ends[1]],
        weights: Some(vec![1.0, half.cos(), 1.0]),
    }
}
