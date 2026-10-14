// SPDX-License-Identifier: GPL-2.0-only
//! ADR0051 private annular embedding evidence, never ordered body validation or publication.
mod access;
mod classification;
mod operation;
mod types;
use super::super::{annular_binding::*, hypothesis::*, two_loop::TwoLoopRegionRequest};
use super::{annular_coverage::*, annular_domains::*, annular_regularity::*, annular_table::*};
use crate::sketch_region_prism::PrismReceipt;
pub(in crate::sketch_region_prism) use access::rebind_bound_annular_embedding;
pub(in crate::sketch_region_prism) use operation::inspect_bound_annular_embedding;
pub(in crate::sketch_region_prism) use types::*;

#[cfg(test)]
mod cancellation_tests;
#[cfg(test)]
mod tests;
