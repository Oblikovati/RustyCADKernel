// SPDX-License-Identifier: GPL-2.0-only
//! Preserve typed child classes; unavailable coverage is never reclassified as completed empty.
use super::{admission::*, diagnostics::*, progress::*};
use crate::sketch_region::{ArrangementCause, ArrangementCompletion};
use crate::sketch_region_candidates::CandidateCause;
use crate::sketch_region_certification::CensusCause;
use crate::sketch_region_contact_transport::ContactTransportStop;
pub(super) fn preparation_class(cause: &OwnedPreparationCause) -> OwnedPreparationClass {
    use OwnedPreparationCause::*;
    match cause {
        Admission(cause) => admission_class(cause),
        Native(_) => OwnedPreparationClass::Failed,
        Census(cause) => census_class(cause),
        Contact(ContactTransportStop::Cancelled { .. }) => OwnedPreparationClass::Cancelled,
        Contact(ContactTransportStop::Policy { .. }) => OwnedPreparationClass::Failed,
        StageUnavailable { class } => *class,
        PrismUnavailable(_) | MissingLocalState { .. } => OwnedPreparationClass::Incomplete,
        Callback => OwnedPreparationClass::Cancelled,
    }
}
fn admission_class(cause: &OwnedPrismAdmissionCause) -> OwnedPreparationClass {
    match cause {
        OwnedPrismAdmissionCause::EmptyInput => OwnedPreparationClass::Declined,
        OwnedPrismAdmissionCause::MultipleBoundaries { .. }
        | OwnedPrismAdmissionCause::OpenBoundary { .. }
        | OwnedPrismAdmissionCause::UnsupportedVersion { .. } => OwnedPreparationClass::Incomplete,
        _ => OwnedPreparationClass::Failed,
    }
}
pub(super) fn arrangement_class(outcome: &ArrangementCompletion) -> OwnedPreparationClass {
    match outcome {
        ArrangementCompletion::Failed(_) => OwnedPreparationClass::Failed,
        ArrangementCompletion::Cancelled(_) => OwnedPreparationClass::Cancelled,
        ArrangementCompletion::Declined(_) => OwnedPreparationClass::Declined,
        _ => OwnedPreparationClass::Incomplete,
    }
}
pub(super) fn callback_class(cause: &OwnedPrismCallbackCause) -> OwnedPreparationClass {
    match cause {
        OwnedPrismCallbackCause::Cancelled => OwnedPreparationClass::Cancelled,
        OwnedPrismCallbackCause::Failed(_) => OwnedPreparationClass::Failed,
        OwnedPrismCallbackCause::Exhausted => OwnedPreparationClass::Incomplete,
    }
}
fn census_class(cause: &CensusCause) -> OwnedPreparationClass {
    match cause {
        CensusCause::SourceCancelled { .. } | CensusCause::Cancelled { .. } => {
            OwnedPreparationClass::Cancelled
        }
        CensusCause::Policy(_) => OwnedPreparationClass::Failed,
        CensusCause::Capture(cause) => candidate_class(cause),
        CensusCause::Input(
            ArrangementCause::Failure { .. } | ArrangementCause::MappedReport { .. },
        ) => OwnedPreparationClass::Failed,
        _ => OwnedPreparationClass::Incomplete,
    }
}
pub(super) fn candidate_class(cause: &CandidateCause) -> OwnedPreparationClass {
    match cause {
        CandidateCause::Mapped(_) | CandidateCause::MappedReport(_) => {
            OwnedPreparationClass::Failed
        }
        CandidateCause::Cancelled(_) => OwnedPreparationClass::Cancelled,
        _ => OwnedPreparationClass::Incomplete,
    }
}
