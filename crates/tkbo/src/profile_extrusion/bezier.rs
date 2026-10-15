// SPDX-License-Identifier: GPL-2.0-only
//! Exact-closure Bézier pieces of each loop for the certified winding classifier. Arcs and
//! circles split into proper rational quadratic pieces of at most a quarter turn; authored
//! vertices are the exact piece ends, and each split point is computed once and shared.
use super::admitted::AdmittedProfile;
use super::loop_geometry::{UseGeometry, use_geometry};
use super::loops::{LoopUse, ProfileLoop};
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
        arc @ UseGeometry::Arc { .. } => arc_pieces(arc),
    }
}

/// Every piece of a closed loop, in traversal order; consecutive ends are bitwise equal.
pub(crate) fn loop_pieces(profile: &AdmittedProfile, closed: &ProfileLoop) -> Vec<BezierPiece> {
    closed
        .uses
        .iter()
        .flat_map(|used| use_pieces(profile, *used))
        .collect()
}

fn arc_pieces(arc: UseGeometry) -> Vec<BezierPiece> {
    let UseGeometry::Arc {
        carrier,
        start_angle,
        sweep,
        start,
        end,
    } = arc
    else {
        unreachable!("arc_pieces receives arcs only")
    };
    let count = (sweep.abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let step = sweep / count as f64;
    let mut ends: Vec<[f64; 2]> = (0..=count)
        .map(|index| {
            on_circle(
                carrier.center,
                carrier.radius,
                start_angle + step * index as f64,
            )
        })
        .collect();
    (ends[0], ends[count]) = (start, end);
    (0..count)
        .map(|index| {
            let middle_angle = start_angle + step * (index as f64 + 0.5);
            let half = 0.5 * step.abs();
            let corner = on_circle(carrier.center, carrier.radius / half.cos(), middle_angle);
            BezierPiece {
                controls: vec![ends[index], corner, ends[index + 1]],
                weights: Some(vec![1.0, half.cos(), 1.0]),
            }
        })
        .collect()
}
