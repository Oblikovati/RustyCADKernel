// SPDX-License-Identifier: GPL-2.0-only
//! Private owned request preparation; no operation completion authority.
pub(super) mod names;

mod admission;
mod classification;
mod construction;
mod diagnostic_access;
mod diagnostics;
pub(crate) use admission::{DeclaredPlaneComponent, OwnedPrismAdmissionCause};
pub(crate) use diagnostic_access::{PreparationDiagnosticView, RetainedPreparationStage};
mod partition;
mod pipeline;
mod progress;
mod request;
mod stages;
pub(crate) use diagnostics::{
    OwnedPreparationCause, OwnedPreparationClass, OwnedPrismPreparationFailure,
    PreparedPolynomialPrism,
};
pub(crate) use pipeline::prepare_owned_polynomial_prism;
pub(crate) use progress::{
    OwnedPrismCallbackCause, OwnedPrismCallbackStop, OwnedPrismCheckpoint, OwnedPrismPhase,
    OwnedPrismProgress,
};
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

#[cfg(test)]
mod diagnostic_access_tests;

pub(crate) mod request_replay;

#[cfg(test)]
mod diagnostic_nested_tests;
#[cfg(test)]
mod diagnostic_projection_tests;

#[cfg(test)]
mod diagnostic_runtime_tests;

#[cfg(test)]
mod diagnostic_codec_tests;
#[cfg(test)]
pub(in crate::sketch_region_prism) mod publication_diagnostic_tests;
mod resolver;

#[cfg(test)]
pub(in crate::sketch_region_prism) use resolver::fixture::{
    ResolvedPrismFixture, default_resolved_prism_fixture, resolved_prism_fixture,
};

/// Sealed local refusal from the actual resolver publication invocation.
pub(in crate::sketch_region_prism) use resolver::InvokedPrismPublicationFailure;
