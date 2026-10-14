// SPDX-License-Identifier: GPL-2.0-only
//! ADR0051 private annular embedding evidence, never ordered body validation or publication.
mod operation;
mod types;
use super::super::{annular_binding::*, hypothesis::*, two_loop::TwoLoopRegionRequest};
use super::{annular_coverage::*, annular_domains::*, annular_regularity::*, annular_table::*};
use crate::sketch_region_prism::PrismReceipt;
pub(super) use operation::inspect_bound_annular_embedding;
pub(super) use types::*;

#[cfg(test)]
mod tests;
