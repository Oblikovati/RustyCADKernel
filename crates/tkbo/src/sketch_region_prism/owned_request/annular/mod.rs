// SPDX-License-Identifier: GPL-2.0-only
//! Owned annular preparation: two explicit boundary roles through the existing general pipeline.
mod access;
mod admission;
mod pipeline;
mod request;
mod retained;
mod roles;
mod stages;
pub(in crate::sketch_region_prism) use admission::AnnularAdmissionCause;
pub(in crate::sketch_region_prism) use pipeline::prepare_owned_annular_prism;
pub(in crate::sketch_region_prism) use request::*;
pub(in crate::sketch_region_prism) use retained::{
    AnnularPreparationCause, AnnularPreparationFailure, PreparedAnnularPrism,
};

use super::admission::OwnedPrismAdmissionCause;
use super::construction::{ConstructedPolynomialSource, construct_owned_polynomial_as};
use super::diagnostic_access::RetainedPreparationStage;
use super::diagnostics::{OwnedPreparationCause, OwnedPreparationClass};
use super::names::CallerOperationId;
use super::progress::*;
use super::request::OwnedPolynomialBoundary;
use crate::sketch_region::{ArrangementSource, RegionSourceId};
use crate::sketch_region_certification::{BoundaryCensus, CensusPolicy};
use crate::sketch_region_contact_transport::{ContactTransportPolicy, PairedContactTransaction};
use crate::sketch_region_partition_bindings::PartitionBoundOutputs;
use crate::sketch_region_prism::PrismReceipt;
use crate::sketch_region_prism::region_certificate::two_loop::{
    TwoLoopCandidateBinding, TwoLoopRegionIdentities, TwoLoopRegionOptions, TwoLoopRegionRequest,
    TwoLoopWireBinding,
};
use crate::sketch_region_source_conformance::SourceConformancePolicy;
use retained::*;
use rustycad_tkmath::{gp::Plane3, predicates::roots::RootIsolationOptions};

#[cfg(test)]
pub(in crate::sketch_region_prism) mod tests;
