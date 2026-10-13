// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{RootIsolation, signs::RootSignEvaluation};
/// Retained evidence from actual winding evaluation; a complete proof also needs its owned descriptor.
#[derive(Clone, Debug)]
pub enum CapturedWindingEvidence {
    /// Actual supporting roots, aligned ray signs and evaluated local contributions.
    Normal {
        /// Actual source isolation and its aligned longitudinal signs.
        evaluated: RootSignEvaluation,
        /// Aligned local crossings; None means no contribution was evaluated.
        contributions: Vec<Option<i32>>,
    },
    /// Supporting-line coincidence evaluated using the actual directed polynomial.
    Coincident {
        /// Actual directed-polynomial isolation on the coincident supporting line.
        directed: RootIsolation,
    },
    /// Both projected normal and directed polynomials are exactly zero.
    ZeroDirected,
}

pub(super) fn normal_result(
    incidence: super::BezierWinding,
    evaluated: RootSignEvaluation,
    contributions: Vec<Option<i32>>,
) -> (super::BezierWinding, CapturedWindingEvidence, u16) {
    let work = evaluated.isolation().bisections();
    (
        incidence,
        CapturedWindingEvidence::Normal {
            evaluated,
            contributions,
        },
        work,
    )
}
pub(super) fn coincident_result(
    directed: RootIsolation,
) -> (super::BezierWinding, CapturedWindingEvidence, u16) {
    let incidence = if directed.roots().is_empty() {
        super::BezierWinding::Clear(0)
    } else {
        super::BezierWinding::Boundary
    };
    let work = directed.bisections();
    (
        incidence,
        CapturedWindingEvidence::Coincident { directed },
        work,
    )
}
