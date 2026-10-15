// SPDX-License-Identifier: GPL-2.0-only
//! ADR0062 follow-up: owned annular preparation refusal. The actual typed admission, role and
//! stage causes are copied with their operands; retained partial transactions and unprojected
//! nested families are explicit gaps, and the exact original Box is kept beside them.
use super::preparation::{operation_class, owned_callback, owned_cause, unprojected_cause};
use super::*;
use crate::sketch_region_prism::owned_request::annular::{
    AnnularAdmissionCause, AnnularPreparationCause, AnnularPreparationFailure,
    OwnedAnnularPrismRequest,
};

/// Owned annular cause; `Stage` reuses the single-boundary owned cause vocabulary unchanged.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum OwnedAnnularPreparationCause {
    Admission(AnnularAdmissionCause),
    Role {
        required: &'static str,
        actual: usize,
        expected: usize,
    },
    Stage(OwnedPreparationDiagnosticCause),
}
/// Immutable refusal evidence: full original request, outer phase, original class and gaps.
pub(in crate::sketch_region_prism) struct OwnedAnnularPreparationDiagnostic {
    pub(super) request: OwnedAnnularPrismRequest,
    pub(super) phase: OwnedPrismPhase,
    pub(super) class: PrismDiagnosticClass,
    pub(super) cause: OwnedAnnularPreparationCause,
    pub(super) callback: Option<OwnedPreparationCallback>,
    pub(super) gaps: Vec<PrismDiagnosticGap>,
}
/// Only the diagnostic is publishable; `Incomplete` keeps the actual local failure for inspection.
pub(in crate::sketch_region_prism) enum AnnularPreparationRefusalProjection {
    Projected(Box<OwnedAnnularPreparationDiagnostic>),
    Incomplete {
        diagnostic: Box<OwnedAnnularPreparationDiagnostic>,
        original: Box<AnnularPreparationFailure>,
    },
}
/// Project an actual annular preparation refusal; `project_annular_preparation_refusal(failure)`.
/// Projection coverage never changes the original operation class.
pub(in crate::sketch_region_prism) fn project_annular_preparation_refusal(
    original: Box<AnnularPreparationFailure>,
) -> AnnularPreparationRefusalProjection {
    let diagnostic = Box::new(capture(&original));
    if diagnostic.gaps.is_empty() {
        return AnnularPreparationRefusalProjection::Projected(diagnostic);
    }
    AnnularPreparationRefusalProjection::Incomplete {
        diagnostic,
        original,
    }
}
fn capture(original: &AnnularPreparationFailure) -> OwnedAnnularPreparationDiagnostic {
    OwnedAnnularPreparationDiagnostic {
        request: original.request().clone(),
        phase: original.phase,
        class: operation_class(original.class),
        cause: cause(&original.cause),
        callback: original.callback.as_ref().map(owned_callback),
        gaps: gaps(original),
    }
}
fn cause(value: &AnnularPreparationCause) -> OwnedAnnularPreparationCause {
    match value {
        AnnularPreparationCause::Admission(cause) => {
            OwnedAnnularPreparationCause::Admission(cause.clone())
        }
        AnnularPreparationCause::Role {
            required,
            actual,
            expected,
        } => OwnedAnnularPreparationCause::Role {
            required,
            actual: *actual,
            expected: *expected,
        },
        AnnularPreparationCause::Stage(cause) => {
            OwnedAnnularPreparationCause::Stage(owned_cause(cause))
        }
    }
}
fn gaps(original: &AnnularPreparationFailure) -> Vec<PrismDiagnosticGap> {
    let mut gaps = Vec::new();
    let stage = match &original.cause {
        AnnularPreparationCause::Stage(cause) => Some(cause),
        _ => None,
    };
    if let Some(payload) = stage.and_then(unprojected_cause) {
        gaps.push(PrismDiagnosticGap::PreparationCause(payload));
    }
    let owners = original.retained_stages();
    if !owners.is_empty() {
        gaps.push(PrismDiagnosticGap::OuterOwners(owners));
    }
    let callback_cause = matches!(stage, Some(OwnedPreparationCause::Callback));
    if callback_cause && original.callback.is_none() {
        gaps.push(PrismDiagnosticGap::MissingCallbackEvidence);
    }
    gaps
}
impl OwnedAnnularPreparationDiagnostic {
    /// Read the full original request; `diagnostic.request().regions.annulus`.
    pub(in crate::sketch_region_prism) fn request(&self) -> &OwnedAnnularPrismRequest {
        &self.request
    }
    /// Read the outer preparation phase; `diagnostic.phase()`.
    pub(in crate::sketch_region_prism) fn phase(&self) -> OwnedPrismPhase {
        self.phase
    }
    /// Read the unchanged original class; `diagnostic.class()` is separate from coverage.
    pub(in crate::sketch_region_prism) fn class(&self) -> PrismDiagnosticClass {
        self.class
    }
    /// Read the owned typed cause; `diagnostic.cause()`.
    pub(in crate::sketch_region_prism) fn cause(&self) -> &OwnedAnnularPreparationCause {
        &self.cause
    }
    /// Read exact callback evidence; `diagnostic.callback()`.
    pub(in crate::sketch_region_prism) fn callback(&self) -> Option<&OwnedPreparationCallback> {
        self.callback.as_ref()
    }
    /// Read ordered explicit gaps; empty means the actual cause is owned losslessly.
    pub(in crate::sketch_region_prism) fn gaps(&self) -> &[PrismDiagnosticGap] {
        &self.gaps
    }
    /// Complete exactly when no gap remains; `diagnostic.coverage()`.
    pub(in crate::sketch_region_prism) fn coverage(&self) -> DiagnosticProjectionCoverage {
        match self.gaps.is_empty() {
            true => DiagnosticProjectionCoverage::Complete,
            false => DiagnosticProjectionCoverage::Incomplete,
        }
    }
}

#[cfg(test)]
#[path = "annular_preparation_tests.rs"]
mod tests;
