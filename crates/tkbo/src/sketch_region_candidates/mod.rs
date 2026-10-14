// SPDX-License-Identifier: GPL-2.0-only
//! Private candidate diagnostics preserve mapped generation without certifying search.
mod ancestry;
mod boundaries;
mod captured_store;
mod classification;
mod generation;
mod outcome;
pub(crate) use classification::classify_candidates;
pub(crate) use generation::extract_candidates;
pub(crate) use outcome::*;
#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
mod native_tests;

#[cfg(test)]
mod coverage_tests;

pub(crate) use boundaries::capture_segment as capture_boundary_segment;

#[cfg(test)]
mod captured_tests;

pub(crate) use generation::composed_copy_relation;

pub(crate) use ancestry::append_unique as append_original_parents;
