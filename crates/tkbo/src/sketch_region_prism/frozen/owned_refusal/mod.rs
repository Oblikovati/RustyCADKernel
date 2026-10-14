// SPDX-License-Identifier: GPL-2.0-only
//! ADR0043: optional diagnostic projection; incomplete projection retains the complete local cause.
mod binding;
mod candidate;
mod embedding;
mod frozen_diagnostic;
mod identity;
mod join;
mod region;
mod source;
mod types;
mod views;
use super::{proof_join::*, *};
use crate::sketch_region_prism::region_certificate::{current_binding::*, embedding::*, *};
use crate::sketch_region_prism::surface_charts::PrismSurfaceCause;
use crate::sketch_region_prism::{PrismCause, PrismCompletion, PrismPhase};
use crate::sketch_region_source_conformance::SourceConformanceCause;
pub(crate) use identity::PrismRefusalIdentityContext;
pub(crate) use join::project_join_refusal;
use rustycad_tkernel::standard::Failure;
pub(crate) use types::*;
#[cfg(test)]
mod tests;
