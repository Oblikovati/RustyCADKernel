// SPDX-License-Identifier: GPL-2.0-only
//! Actual bound cap annuli and source-specific sides; no reuse of the V1 Jordan-cap meaning.
mod capture;
mod regularity_sources;
mod selection;
mod types;
use super::super::{
    annular_binding::*,
    hypothesis::{HypothesisCheck, holds, need, require},
};
use super::{PrismAuthoredTrimUse, PrismImageRole, PrismTrimRole, trim_incidence};
use crate::sketch_region::{RegionObservedContext, RegionSourceId};
pub(super) use capture::capture_annular_domains;
pub(super) use regularity_sources::bound_regularity_operands;
pub(super) use types::*;

#[cfg(test)]
mod refusal_tests;
#[cfg(test)]
mod tests;
