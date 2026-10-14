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

mod resolver;

#[cfg(test)]
pub(in crate::sketch_region_prism) use resolver::fixture::{
    ResolvedPrismFixture, default_resolved_prism_fixture, resolved_prism_fixture,
};

/// Sealed local refusal from the actual resolver publication invocation.
pub(in crate::sketch_region_prism) use resolver::InvokedPrismPublicationFailure;
