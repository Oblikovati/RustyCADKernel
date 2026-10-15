// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0066 part C owned profile preparation: any number of polynomial or rational edges in
//! authored loops, through the shared general stages, then the general loop region proof.
mod admission;
mod certify;
mod construction;
mod pipeline;
mod regions;
mod request;
pub(in crate::sketch_region_prism) use admission::{
    ProfileAdmissionCause, ProfileEdgeCause, ProfileLoopCause,
};
pub(crate) use certify::{
    ProfileGraphInput, ProfileProofRefusal, ProofEdge, certify_profile_graph,
};
pub(in crate::sketch_region_prism) use construction::ConstructedProfileEdge;
pub(in crate::sketch_region_prism) use pipeline::{
    PreparedProfile, ProfilePreparationCause, ProfilePreparationFailure, prepare_owned_profile,
};
pub(in crate::sketch_region_prism) use request::*;

use super::admission::OwnedPrismAdmissionCause;
use super::diagnostics::{OwnedPreparationCause, OwnedPreparationClass};
use super::names::{CallerBoundaryId, CallerOperationId, CallerVertexId};
use super::progress::*;
use super::request::CallerSourceRevision;
use crate::sketch_region::{ArrangementSource, RegionSourceId};
use crate::sketch_region_certification::{BoundaryCensus, CensusPolicy};
use crate::sketch_region_contact_transport::{ContactTransportPolicy, PairedContactTransaction};
use crate::sketch_region_prism::region_certificate::loop_region::LoopRegionOptions;
use crate::sketch_region_source_conformance::SourceConformancePolicy;
use rustycad_tkmath::gp::Plane3;

#[cfg(test)]
mod tests;
