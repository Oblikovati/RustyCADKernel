// SPDX-License-Identifier: GPL-2.0-only
//! Pick results: exact hits on B-rep faces in a total, deterministic order.
use crate::topology::FaceIndex;
use std::cmp::Ordering;

/// One exact intersection of the ray with a face.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickHit {
    /// The stable index of the hit face.
    pub face: FaceIndex,
    /// The distance from the ray origin along the unit direction.
    pub distance: f64,
    /// The surface parameters of the hit on the face.
    pub uv: [f64; 2],
    /// The hit point in model coordinates.
    pub point: [f64; 3],
    /// True when the hit lies on the face boundary (within the edge tolerances),
    /// so neighbouring faces can report the same point.
    pub on_boundary: bool,
}

impl PickHit {
    /// The pick order: distance, then face index, then u, then v, all compared as
    /// totally ordered floats, so ties never depend on evaluation order.
    pub fn pick_order(&self, other: &Self) -> Ordering {
        self.distance
            .total_cmp(&other.distance)
            .then(self.face.cmp(&other.face))
            .then(self.uv[0].total_cmp(&other.uv[0]))
            .then(self.uv[1].total_cmp(&other.uv[1]))
    }
}

/// How the hits were found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickCompleteness {
    /// Every face is a plane, cylinder, cone, sphere or torus, intersected in closed
    /// form: no crossing of the ray with a face can be missing.
    Analytic,
    /// Some face is free-form and was intersected from sampled candidates refined
    /// on the exact surface; a crossing can be missing.
    SampledCandidates,
}

/// The hits of a ray on a body, nearest first.
#[derive(Clone, Debug, PartialEq)]
pub struct PickResult {
    /// The hits in [`PickHit::pick_order`].
    pub hits: Vec<PickHit>,
    /// Planar faces whose plane contains the ray (within the confusion tolerance):
    /// their overlap with the ray is a segment, not a point, so they give no hit.
    pub coplanar_faces: Vec<FaceIndex>,
    /// Whether the hit list is complete by construction.
    pub completeness: PickCompleteness,
}

impl PickResult {
    /// The nearest hit, if any; `result.nearest()`.
    pub fn nearest(&self) -> Option<&PickHit> {
        self.hits.first()
    }
}

// Published pick results are read from any thread (AGENTS.md ownership rules).
const _: fn() = || {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<PickResult>();
};
