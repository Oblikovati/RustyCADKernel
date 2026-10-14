// SPDX-License-Identifier: GPL-2.0-only
//! ADR0059: exact ordered validation over one sealed annular domain.
mod access;
mod adapters;
mod checks;
mod error;
mod operation;
mod progress;
mod replay;
mod types;
use super::super::{
    annular_join::*,
    annular_orientation::*,
    topology::{self, *},
    *,
};
use super::{
    OrderedPrismFailureClass, OrderedPrismStage, OrderedPrismStageStatus, OrderedPrismStageTrace,
};
use crate::sketch_region_prism::{
    exact_tolerance::{annular::*, *},
    region_certificate::embedding::*,
};
pub(in crate::sketch_region_prism) use error::*;
pub(in crate::sketch_region_prism) use operation::validate_annular_prism;
pub(in crate::sketch_region_prism) use progress::*;
pub(in crate::sketch_region_prism) use types::*;
type ValidationCheck<T> = Result<T, ValidationStop>;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod cancellation_tests;
#[cfg(test)]
mod replay_tests;
