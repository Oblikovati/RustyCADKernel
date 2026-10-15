// SPDX-License-Identifier: GPL-2.0-only
//! Loops of caller curves. Joins come from shared vertex ids only: every authored vertex
//! must end at least two curves (a dangling end is an open profile). Which curves bound
//! which face is decided by the certified plane graph proof (ADR 0084), never here.
use super::admitted::{AdmittedCurve, AdmittedProfile};
use super::decline::DeclineCause;
use super::profile::{ProfileCurveId, ProfileVertexId};
use std::collections::BTreeMap;

/// One directed use of a caller curve inside a loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopUse {
    /// The caller curve.
    pub curve: ProfileCurveId,
    /// Traversed from the authored end to the authored start.
    pub reversed: bool,
}

/// A closed loop of directed curve uses in traversal order: counterclockwise about the
/// plane normal for an outer loop, clockwise for a hole (as in the face). It starts at its
/// curve that comes first in caller order, so the value is deterministic for a given input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileLoop {
    /// Directed uses in traversal order.
    pub uses: Vec<LoopUse>,
}

impl ProfileLoop {
    /// Rotate so that the use of the earliest caller curve comes first; `positions` maps a
    /// curve id to its caller index.
    pub(crate) fn starting_first(mut self, position: impl Fn(ProfileCurveId) -> usize) -> Self {
        let first = (0..self.uses.len()).min_by_key(|i| position(self.uses[*i].curve));
        self.uses.rotate_left(first.unwrap_or(0));
        self
    }
}

/// Every authored vertex must end at least two curves; `require_closed(&profile)?`.
/// Branches (three or more ends) are admitted: the plane graph proof orders them.
pub(crate) fn require_closed(profile: &AdmittedProfile) -> Result<(), DeclineCause> {
    let mut uses: BTreeMap<ProfileVertexId, usize> = BTreeMap::new();
    let ends = profile
        .curves
        .iter()
        .filter_map(AdmittedCurve::ends)
        .flatten();
    for vertex in ends {
        *uses.entry(vertex).or_default() += 1;
    }
    match uses.into_iter().find(|(_, count)| *count < 2) {
        Some((vertex, uses)) => Err(DeclineCause::OpenProfile { vertex, uses }),
        None => require_no_bridge(profile),
    }
}

/// A curve whose removal disconnects its ends bounds no face on either side (a bridge
/// between loops); the region pipeline does not admit it (ADR 0084).
fn require_no_bridge(profile: &AdmittedProfile) -> Result<(), DeclineCause> {
    let edges: Vec<(ProfileCurveId, [ProfileVertexId; 2])> = profile
        .curves
        .iter()
        .filter_map(|curve| Some((curve.id(), curve.ends()?)))
        .collect();
    for (skip, (curve, [start, end])) in edges.iter().enumerate() {
        if start != end && !connected(&edges, skip, *start, *end) {
            return Err(DeclineCause::BridgeCurve { curve: *curve });
        }
    }
    Ok(())
}

/// Whether `to` is reachable from `from` without the edge at index `skip`.
fn connected(
    edges: &[(ProfileCurveId, [ProfileVertexId; 2])],
    skip: usize,
    from: ProfileVertexId,
    to: ProfileVertexId,
) -> bool {
    let mut reached = std::collections::BTreeSet::from([from]);
    let mut frontier = vec![from];
    while let Some(vertex) = frontier.pop() {
        let around = edges
            .iter()
            .enumerate()
            .filter(|(i, (_, ends))| *i != skip && ends.contains(&vertex));
        for (_, (_, ends)) in around {
            let other = if ends[0] == vertex { ends[1] } else { ends[0] };
            if reached.insert(other) {
                frontier.push(other);
            }
        }
    }
    reached.contains(&to)
}
