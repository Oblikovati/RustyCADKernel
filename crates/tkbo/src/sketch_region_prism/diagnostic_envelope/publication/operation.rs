// SPDX-License-Identifier: GPL-2.0-only
//! Unsupported projection keeps the exact original outer Box for operation-local inspection.
use super::*;
/// Project only an actually issued refusal; `project_invoked_publication_refusal(packet)` cannot pair detached headers.
pub(in crate::sketch_region_prism) fn project_invoked_publication_refusal<'a>(
    packet: Box<InvokedPrismPublicationFailure<'a>>,
) -> InvokedPublicationRefusalProjection<'a> {
    let context = OwnedPrismDiagnosticContext::from_invocation(&packet);
    let mut gaps = admission::gaps(&packet);
    let bindings = packet.bindings();
    let original = match history::take(packet.into_failure(), gaps.is_empty()) {
        Ok((header, history)) => return owned_history(context, header, history),
        Err(original) => original,
    };
    let cause = projection::outer(&original, bindings, &mut gaps);
    let diagnostic = diagnostic(context, &original, cause, gaps);
    let original = (!diagnostic.gaps.is_empty()).then_some(original);
    InvokedPublicationRefusalProjection {
        diagnostic,
        original,
    }
}
/// A moved complete history finding needs no retained original.
fn owned_history<'a>(
    context: OwnedPrismDiagnosticContext,
    header: history::HistoryHeader,
    history: OwnedPublicationHistory,
) -> InvokedPublicationRefusalProjection<'a> {
    let diagnostic = Box::new(OwnedPrismPublicationDiagnostic {
        context,
        operation: header.operation,
        phase: header.phase,
        class: header.class,
        cause: OwnedPublicationDiagnosticCause::History(Box::new(history)),
        gaps: Vec::new(),
    });
    InvokedPublicationRefusalProjection {
        diagnostic,
        original: None,
    }
}
fn diagnostic(
    context: OwnedPrismDiagnosticContext,
    original: &PrismPublicationFailure<'_>,
    cause: OwnedPublicationDiagnosticCause,
    gaps: Vec<PublicationDiagnosticGap>,
) -> Box<OwnedPrismPublicationDiagnostic> {
    Box::new(OwnedPrismPublicationDiagnostic {
        context,
        operation: original.operation,
        phase: original.phase,
        class: original.class,
        cause,
        gaps,
    })
}
