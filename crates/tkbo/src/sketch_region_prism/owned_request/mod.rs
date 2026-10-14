// SPDX-License-Identifier: GPL-2.0-only
//! Private owned request preparation; no operation completion authority.
pub(super) mod names;

mod admission;
mod classification;
mod construction;
mod diagnostics;
mod partition;
mod pipeline;
mod progress;
mod request;
mod stages;
pub(crate) use diagnostics::{
    OwnedPreparationClass, OwnedPrismPreparationFailure, PreparedPolynomialPrism,
};
pub(crate) use pipeline::prepare_owned_polynomial_prism;
pub(crate) use progress::{OwnedPrismCheckpoint, OwnedPrismPhase, OwnedPrismProgress};
pub(crate) use request::*;
#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod pipeline_tests;
#[cfg(test)]
mod progress_tests;
#[cfg(test)]
mod test_fixture;

#[cfg(test)]
mod diagnostic_baseline_tests;
