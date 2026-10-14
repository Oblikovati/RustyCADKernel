// SPDX-License-Identifier: GPL-2.0-only
//! Actual correspondence assembly; no public result or map issuance from provisional rows.
mod copied;
mod operands;
mod partition;
mod plan;
mod source;
mod types;
use super::{diagnostics::PreparedPolynomialPrism, names::*};
use crate::sketch_region::{RegionCopyContexts, RegionObservedContext};
use crate::sketch_region_prism::frozen::complete_history::*;
use types::*;
#[cfg(test)]
mod source_tests;

#[cfg(test)]
mod partition_tests;

mod current;
mod generated;
mod selected_copy;

#[cfg(test)]
mod generated_tests;
