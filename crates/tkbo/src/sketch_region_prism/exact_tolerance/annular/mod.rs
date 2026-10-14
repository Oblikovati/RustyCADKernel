// SPDX-License-Identifier: GPL-2.0-only
//! ADR0057 sealed annular exact geometry and separate raw metadata; no body promotion.
mod conformance;
mod failure;
mod progress;
mod replay;
mod source_closures;
mod source_copy;
mod source_reports;
mod source_types;
mod tolerance;
mod types;
mod views;
use super::{joined_progress::*, joined_types::*, tolerance_types::*};
use crate::sketch_region_prism::frozen::{annular_join::AnnularFrozenDomain, *};
use crate::sketch_region_prism::region_certificate::{annular_binding::*, two_loop::*};
pub(in crate::sketch_region_prism) use conformance::certify_annular_prism_conformance;
pub(in crate::sketch_region_prism) use failure::*;
pub(in crate::sketch_region_prism) use source_types::*;
pub(in crate::sketch_region_prism) use tolerance::certify_annular_prism_tolerances;
pub(in crate::sketch_region_prism) use types::*;

#[cfg(test)]
mod cancel_tests;
#[cfg(test)]
mod group_tests;
#[cfg(test)]
mod metadata_tests;
#[cfg(test)]
mod operand_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod source_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod variant_tests;
