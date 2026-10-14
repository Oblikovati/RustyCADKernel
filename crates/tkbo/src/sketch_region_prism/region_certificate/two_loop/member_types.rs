// SPDX-License-Identifier: GPL-2.0-only
//! Authored paths and certified native endpoint roles name directed uses, not parent-set equivalence classes.
use crate::sketch_region::RegionSourceId;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TwoLoopMemberRole {
    Face,
    Wire,
    Boundary,
    NativeStart,
    NativeEnd,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TwoLoopMemberKey {
    pub candidate: u64,
    pub wire: Option<u64>,
    pub role: TwoLoopMemberRole,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TwoLoopOriginalUse {
    pub source: RegionSourceId,
    pub role: TwoLoopMemberRole,
}
#[derive(Debug)]
pub(crate) struct TwoLoopMemberParents {
    pub preserved: Vec<TwoLoopOriginalUse>,
    pub modified: Vec<TwoLoopOriginalUse>,
    pub generated: Vec<TwoLoopOriginalUse>,
}
pub(crate) struct TwoLoopMemberProof {
    pub key: TwoLoopMemberKey,
    pub orientation: Orientation,
    pub placement: MetricEmbedding<3>,
    pub raw_tolerance: Option<f64>,
    pub parents: TwoLoopMemberParents,
}
