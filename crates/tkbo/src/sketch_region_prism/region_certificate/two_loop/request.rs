// SPDX-License-Identifier: GPL-2.0-only
//! Explicit internal identity adapters; authored roles are hypotheses until inspected.
use crate::sketch_region::{ArrangementSource, PartitionOccurrenceIdentity, RegionSourceId};
use crate::sketch_region_contact_transport::PairedContactTransaction;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TwoLoopRegionBehavior {
    NestedPolynomialV2,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TwoLoopRegionOptions {
    pub behavior: TwoLoopRegionBehavior,
    pub winding: RootIsolationOptions,
    /// Aggregate captured structural units, including temporary query inputs and nested rows; not bytes.
    pub maximum_retained_units: usize,
    pub maximum_poles: usize,
}
pub(crate) struct TwoLoopRegionRequest<'inputs, 'bindings> {
    pub bridge: &'inputs PairedContactTransaction,
    pub sources: &'bindings [ArrangementSource<'inputs>],
    pub identities: &'bindings TwoLoopRegionIdentities,
    /// Internal witness only; exact winding must establish inner membership.
    pub inner_interior: [f64; 2],
    /// The actual selected material point, checked against every current candidate.
    pub material_seed: [f64; 2],
    pub requested: TwoLoopRegionOptions,
}
pub(crate) struct TwoLoopRegionIdentities {
    pub operation: u64,
    pub outer: RegionSourceId,
    pub inner: RegionSourceId,
    pub selected: u64,
    pub candidates: Vec<TwoLoopCandidateBinding>,
}
pub(crate) struct TwoLoopCandidateBinding {
    pub name: u64,
    pub actual: PartitionOccurrenceIdentity,
    pub wires: Vec<TwoLoopWireBinding>,
}
pub(crate) struct TwoLoopWireBinding {
    pub name: u64,
    pub actual: PartitionOccurrenceIdentity,
    pub source: RegionSourceId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TwoLoopRegionStage {
    Admission,
    Sources,
    Census,
    Membership,
    Nesting,
    Commit,
}
pub(crate) trait TwoLoopRegionProgress {
    /// Observe bounded proof work; example: `progress.checkpoint(TwoLoopRegionStage::Sources)`.
    fn checkpoint(&mut self, stage: TwoLoopRegionStage) -> Result<bool, Failure>;
}
