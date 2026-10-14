// SPDX-License-Identifier: GPL-2.0-only
//! ADR0058: source/candidate-scoped history; observed subsets do not grant completion.
mod types;
pub(in crate::sketch_region_prism) use types::*;

use super::annular_join::AnnularFrozenDomain;
use super::complete_history::*;
use super::lineage::PrismCopyBindings;
use super::*;
use crate::sketch_region::{RegionObservedContext, RegionSourceId};
use crate::sketch_region_prism::PrismReceipt;
use crate::sketch_region_prism::region_certificate::{
    annular_binding::AnnularPrismBinding, two_loop::*,
};

mod admission;
mod budget;
mod candidates;
mod capture;
mod census;
mod continuation;
mod metadata;
mod names;
mod operation;
mod partition;
mod receipts;
mod replay;
mod result;
mod scopes;
mod sources;
mod sweep_parents;
mod traces;
pub(in crate::sketch_region_prism) use candidates::{
    AnnularCandidateCopy, AnnularCandidateHistory, AnnularCandidateWire,
};
pub(in crate::sketch_region_prism) use capture::AnnularScopeRoster;
pub(in crate::sketch_region_prism) use continuation::{
    AnnularOriginalContinuation, AnnularWireRetirement,
};
pub(in crate::sketch_region_prism) use operation::{
    inspect_annular_prism_history, inspect_bound_annular_prism_history,
    rebind_annular_prism_history, reproject_annular_prism_history,
};
pub(in crate::sketch_region_prism) use result::*;
pub(in crate::sketch_region_prism) use sources::{AnnularClosureWitness, AnnularSourceHistory};
pub(in crate::sketch_region_prism) use sweep_parents::AnnularCapturedSweepParents;
pub(in crate::sketch_region_prism) use traces::AnnularGeneratedTrace;

#[cfg(test)]
mod adversarial_tests;
#[cfg(test)]
mod baseline_tests;
#[cfg(test)]
pub(in crate::sketch_region_prism) mod fixture;
#[cfg(test)]
mod native_refusal_tests;
#[cfg(test)]
pub(super) mod operation_tests;
#[cfg(test)]
mod variant_tests;
