// SPDX-License-Identifier: GPL-2.0-only
//! ADR0061 follow-up: owned annular publication refusal. A refusal whose leaf has an explicit
//! codec is moved whole into the owned diagnostic (typed child failure plus its bytes); any other
//! leaf is an explicit pending payload and the exact original Box is returned beside it.
mod join;
mod project;
mod replay;
mod types;
pub(in crate::sketch_region_prism) use join::*;
pub(in crate::sketch_region_prism) use project::project_annular_publication_refusal;
pub(in crate::sketch_region_prism) use types::*;

use super::history_codec::finding_bytes;
use super::projection::callback as owned_callback;
use super::*;
use crate::sketch_region_prism::frozen::{
    annular_join::{AnnularFrozenCause, AnnularFrozenFailure, AnnularFrozenOptions},
    annular_measures::{AnnularMeasureFailure, annular_measure_failure_bytes},
    ordered_validation::annular::{AnnularValidationFailure, annular_validation_failure_bytes},
};
use crate::sketch_region_prism::publication::annular::{
    AnnularPublicationCause, AnnularPublicationFailure, AnnularPublicationOptions,
};

#[cfg(test)]
mod tests;
