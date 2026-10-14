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
    let original = packet.into_failure();
    let cause = projection::outer(&original, bindings, &mut gaps);
    let diagnostic = diagnostic(context, &original, cause, gaps);
    let original = (!diagnostic.gaps.is_empty()).then_some(original);
    InvokedPublicationRefusalProjection {
        diagnostic,
        original,
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
