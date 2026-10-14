// SPDX-License-Identifier: GPL-2.0-only
//! Unsupported nested causes remain incomplete even when every outer owner field is empty.
use super::*;
/// Project actual preparation evidence; `project_owned_preparation_refusal(failure)` retains local owners on incomplete coverage.
pub(crate) fn project_owned_preparation_refusal(
    original: Box<OwnedPrismPreparationFailure>,
) -> PreparationRefusalProjection {
    let mut diagnostic = capture(&original);
    collect_gaps(&original, &mut diagnostic.gaps);
    if diagnostic.gaps.is_empty() {
        return PreparationRefusalProjection::Projected(Box::new(diagnostic));
    }
    PreparationRefusalProjection::Incomplete {
        diagnostic: Box::new(diagnostic),
        original,
    }
}
fn capture(original: &OwnedPrismPreparationFailure) -> OwnedPrismDiagnostic {
    OwnedPrismDiagnostic {
        context: OwnedPrismDiagnosticContext::from_preparation(original),
        phase: PrismDiagnosticPhase::PreparationBoundary(
            original.diagnostic_context().outer_entry_phase(),
        ),
        class: operation_class(original.class),
        cause: owned_cause(&original.cause),
        callback: original.callback.as_ref().map(owned_callback),
        gaps: Vec::new(),
    }
}
fn operation_class(class: OwnedPreparationClass) -> PrismDiagnosticClass {
    match class {
        OwnedPreparationClass::Declined => PrismDiagnosticClass::Declined,
        OwnedPreparationClass::Incomplete => PrismDiagnosticClass::Incomplete,
        OwnedPreparationClass::Failed => PrismDiagnosticClass::Failed,
        OwnedPreparationClass::Cancelled => PrismDiagnosticClass::Cancelled,
    }
}
fn owned_cause(cause: &OwnedPreparationCause) -> OwnedPreparationDiagnosticCause {
    use OwnedPreparationCause as C;
    use OwnedPreparationDiagnosticCause as O;
    use UnprojectedPreparationPayload as U;
    match cause {
        C::Admission(value) => O::Admission(value.clone()),
        C::Native(value) => O::Native(value.clone()),
        C::Callback => O::Callback,
        C::StageUnavailable { class } => O::StageUnavailable { class: *class },
        C::MissingLocalState { expected } => O::MissingLocalState { expected },
        C::Census(_) => O::Unprojected(U::Census),
        C::Contact(_) => O::Unprojected(U::ContactTransport),
        C::PrismUnavailable(_) => O::Unprojected(U::RegionPrism),
    }
}
fn owned_callback(stop: &OwnedPrismCallbackStop) -> OwnedPreparationCallback {
    let cause = match &stop.cause {
        OwnedPrismCallbackCause::Cancelled => OwnedDiagnosticCallbackCause::Cancelled,
        OwnedPrismCallbackCause::Failed(value) => {
            OwnedDiagnosticCallbackCause::Failed(value.clone())
        }
        OwnedPrismCallbackCause::Exhausted => OwnedDiagnosticCallbackCause::Exhausted,
    };
    OwnedPreparationCallback {
        checkpoint: stop.checkpoint,
        cause,
    }
}
fn collect_gaps(original: &OwnedPrismPreparationFailure, gaps: &mut Vec<PrismDiagnosticGap>) {
    if let Some(payload) = unprojected_cause(&original.cause) {
        gaps.push(PrismDiagnosticGap::PreparationCause(payload));
    }
    let owners = original.diagnostic_context().outer_retained_stages();
    if !owners.is_empty() {
        gaps.push(PrismDiagnosticGap::OuterOwners(owners));
    }
    if matches!(original.cause, OwnedPreparationCause::Callback) && original.callback.is_none() {
        gaps.push(PrismDiagnosticGap::MissingCallbackEvidence);
    }
}
fn unprojected_cause(cause: &OwnedPreparationCause) -> Option<UnprojectedPreparationPayload> {
    use OwnedPreparationCause as C;
    use UnprojectedPreparationPayload as U;
    match cause {
        C::Census(_) => Some(U::Census),
        C::Contact(_) => Some(U::ContactTransport),
        C::PrismUnavailable(_) => Some(U::RegionPrism),
        C::StageUnavailable { .. } => Some(U::StageCompletion),
        C::Admission(_) | C::Native(_) | C::MissingLocalState { .. } | C::Callback => None,
    }
}
