// SPDX-License-Identifier: GPL-2.0-only
//! Preserve existing native/source/query classifications without parsing display strings.
use super::*;
use super::super::{current_binding::PrismBindingFailure, RegionCertificateCause};
use super::super::super::frozen::complete_history::PrismHistoryFailureClass;
impl AnnularBindingFailure<'_> {
    /// Read the retained cause's classification; `failure.classification()` does not promote unsupported evidence.
    pub(in crate::sketch_region_prism) fn classification(&self) -> TwoLoopRefusalClass {
        classify(&self.cause)
    }
}
fn classify(cause: &AnnularBindingCause<'_>) -> TwoLoopRefusalClass {
    use AnnularBindingCause::*;
    match cause {
        Region(failure) => failure.classification(),
        Chart(cause) => chart(cause),
        History(cause) => match cause.failure_class() { PrismHistoryFailureClass::Failed => TwoLoopRefusalClass::Failed, _ => TwoLoopRefusalClass::Incomplete },
        Native(_) | Callback { .. } => TwoLoopRefusalClass::Failed,
        Capture(cause) => classify_region_surface_cause(cause),
        Body(cause) => body(cause),
        Count { .. } => TwoLoopRefusalClass::Incomplete,
        Cancelled(_) => TwoLoopRefusalClass::Cancelled,
        Stopped { callback, .. } => classify(callback),
    }
}
fn chart(cause: &PrismBindingFailure) -> TwoLoopRefusalClass {
    match cause {
        PrismBindingFailure::Region(cause) => region(cause),
        PrismBindingFailure::Classification(cause) => classify_region_candidate_cause(cause),
        PrismBindingFailure::Chart(cause) => classify_region_surface_cause(cause),
        PrismBindingFailure::Body(PrismCompletion::Failed(cause)) => body(cause),
        PrismBindingFailure::Body(PrismCompletion::Cancelled(_)) | PrismBindingFailure::Cancelled => TwoLoopRefusalClass::Cancelled,
        PrismBindingFailure::Body(_) => TwoLoopRefusalClass::Incomplete,
    }
}
fn region(cause: &RegionCertificateCause) -> TwoLoopRefusalClass {
    match cause {
        RegionCertificateCause::Mapped(_) => TwoLoopRefusalClass::Failed,
        RegionCertificateCause::Capture(cause) => classify_region_surface_cause(cause),
        _ => TwoLoopRefusalClass::Incomplete,
    }
}
fn body(cause: &PrismCause) -> TwoLoopRefusalClass {
    match cause {
        PrismCause::Mapped { .. } | PrismCause::ClassificationProgress { .. } => TwoLoopRefusalClass::Failed,
        PrismCause::Classification(cause) => classify_region_candidate_cause(cause),
        _ => TwoLoopRefusalClass::Incomplete,
    }
}
