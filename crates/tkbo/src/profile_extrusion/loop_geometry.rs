// SPDX-License-Identifier: GPL-2.0-only
//! Directed use geometry in plane coordinates and the closed-form signed loop area, a
//! cross-check of the certified area. The plane graph proof decides every topological fact.
use super::admitted::{AdmittedCurve, AdmittedProfile, CircleCarrier};
use super::bezier::BezierPiece;
use super::ellipse::EllipseCarrier;
use super::loops::{LoopUse, ProfileLoop};
use super::plane_math::*;
use super::profile::ArcSense;

/// The carrier of a circular or elliptic arc: the image of the unit circle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Conic {
    Circle(CircleCarrier),
    Ellipse(EllipseCarrier),
}

impl Conic {
    /// Point at (affine) parameter `t`.
    pub(crate) fn point(&self, t: f64) -> [f64; 2] {
        match self {
            Self::Circle(c) => on_circle(c.center, c.radius, t),
            Self::Ellipse(e) => e.point(t),
        }
    }
    /// Image of the unit-circle point `[x, y]`.
    pub(crate) fn image(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        match self {
            Self::Circle(c) => [c.center[0] + c.radius * x, c.center[1] + c.radius * y],
            Self::Ellipse(e) => e.image([x, y]),
        }
    }
    /// Parameter of a point on the carrier.
    fn parameter(&self, p: [f64; 2]) -> f64 {
        match self {
            Self::Circle(c) => polar_angle(c.center, p),
            Self::Ellipse(e) => e.parameter(p),
        }
    }
    /// Green's term of the arc from `start` to `end` sweeping `sweep`.
    fn green_term(&self, start: [f64; 2], end: [f64; 2], sweep: f64) -> f64 {
        match self {
            Self::Circle(c) => {
                let chord = cross(c.center, sub2(end, start));
                0.5 * (chord + c.radius * c.radius * sweep)
            }
            Self::Ellipse(e) => e.green_term(start, end, sweep),
        }
    }
}

/// One directed use resolved to its traversal geometry.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UseGeometry {
    Segment {
        start: [f64; 2],
        end: [f64; 2],
    },
    /// `sweep` is signed in traversal direction (positive counterclockwise).
    Arc {
        carrier: Conic,
        start_angle: f64,
        sweep: f64,
        start: [f64; 2],
        end: [f64; 2],
    },
    /// Bézier pieces in traversal order; no closed-form area.
    Spline {
        pieces: Vec<BezierPiece>,
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
    let points = |ends: [_; 2]| ends.map(|vertex| profile.point(vertex));
    match curve {
        AdmittedCurve::Line { ends, .. } => {
            let [start, end] = points(*ends);
            UseGeometry::Segment { start, end }
        }
        AdmittedCurve::Arc {
            ends,
            carrier,
            sense,
            ..
        } => arc_geometry(Conic::Circle(*carrier), points(*ends), *sense),
        AdmittedCurve::EllipticArc {
            ends,
            carrier,
            sense,
            ..
        } => arc_geometry(Conic::Ellipse(*carrier), points(*ends), *sense),
        AdmittedCurve::Circle { carrier, .. } => closed_geometry(Conic::Circle(*carrier)),
        AdmittedCurve::Ellipse { carrier, .. } => closed_geometry(Conic::Ellipse(*carrier)),
        AdmittedCurve::Spline { spline, .. } => UseGeometry::Spline {
            pieces: spline.bezier_pieces(),
        },
    }
}

/// A full turn: coincident ends at parameter 0, counterclockwise.
fn closed_geometry(carrier: Conic) -> UseGeometry {
    let start = carrier.point(0.0);
    arc_geometry(carrier, [start, start], ArcSense::CounterClockwise)
}

fn arc_geometry(carrier: Conic, [start, end]: [[f64; 2]; 2], sense: ArcSense) -> UseGeometry {
    let start_angle = carrier.parameter(start);
    let sweep = signed_sweep(start_angle, carrier.parameter(end), sense);
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
        UseGeometry::Spline { pieces } => UseGeometry::Spline {
            pieces: pieces
                .into_iter()
                .rev()
                .map(BezierPiece::reversed)
                .collect(),
        },
    }
}

fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

/// Green's-theorem contribution `1/2 * integral (x dy - y dx)` of one directed use, `None`
/// for a spline.
fn green_term(geometry: &UseGeometry) -> Option<f64> {
    match geometry {
        UseGeometry::Segment { start, end } => Some(0.5 * cross(*start, *end)),
        UseGeometry::Arc {
            carrier,
            sweep,
            start,
            end,
            ..
        } => Some(carrier.green_term(*start, *end, *sweep)),
        UseGeometry::Spline { .. } => None,
    }
}

/// Scale of one use for rounding estimates: its Green term's magnitude, or for a spline
/// the sum over pieces of the largest control distance times the control polygon length.
fn use_scale(geometry: &UseGeometry) -> f64 {
    match geometry {
        UseGeometry::Spline { pieces } => pieces.iter().map(BezierPiece::green_scale).sum(),
        other => green_term(other).map_or(0.0, f64::abs),
    }
}

/// Closed-form signed area of a loop (positive when counterclockwise), `None` when a spline
/// bounds it.
pub(crate) fn signed_area(profile: &AdmittedProfile, closed: &ProfileLoop) -> Option<f64> {
    let geometry = closed.uses.iter().map(|used| use_geometry(profile, *used));
    geometry.map(|g| green_term(&g)).sum()
}

/// Summed rounding scale of a loop's uses (see `use_scale`).
pub(crate) fn loop_scale(profile: &AdmittedProfile, closed: &ProfileLoop) -> f64 {
    let geometry = closed.uses.iter().map(|used| use_geometry(profile, *used));
    geometry.map(|g| use_scale(&g)).sum()
}
