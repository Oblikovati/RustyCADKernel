// SPDX-License-Identifier: GPL-2.0-only
//! ADR0061: one named annular prism from one sealed join, ordered validation, complete V2 history
//! and conditional measures; V1 single-boundary publication remains selected by its own entry.
mod operation;
mod replay;
mod request;
mod result;
mod stages;
pub(in crate::sketch_region_prism) use operation::publish_annular_prism;
pub(in crate::sketch_region_prism) use request::*;
pub(in crate::sketch_region_prism) use result::*;

use super::progress::*;
use super::stage_error::PrismPublicationClass;
use crate::sketch_region_prism::PrismReceipt;
use crate::sketch_region_prism::frozen::{
    AuthoredPrismId, PrismPublicationBindings,
    annular_history::*,
    annular_join::*,
    annular_measures::*,
    complete_history::{PrismHistoryFailureClass, PrismHistoryOptions, PrismHistoryPhase},
    joined_measures::JoinedMeasureFailureClass,
    lineage::PrismCopyBindings,
    ordered_validation::{OrderedPrismFailureClass, annular::*},
};
use crate::sketch_region_prism::region_certificate::{
    embedding::AnnularEmbeddingEvidence,
    two_loop::{TwoLoopRefusalClass, TwoLoopRegionRequest},
};

#[cfg(test)]
pub(in crate::sketch_region_prism) mod tests;
