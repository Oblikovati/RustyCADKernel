// SPDX-License-Identifier: GPL-2.0-only
//! Private candidate diagnostics preserve mapped generation without certifying search.
mod ancestry;
mod boundaries;
mod classification;
mod generation;
mod outcome;
pub(crate) use classification::classify_candidates;
pub(crate) use generation::extract_candidates;
pub(crate) use outcome::*;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod native_tests;

#[cfg(test)]
mod coverage_tests;
