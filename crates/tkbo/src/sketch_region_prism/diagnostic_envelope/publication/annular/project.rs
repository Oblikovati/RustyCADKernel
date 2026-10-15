// SPDX-License-Identifier: GPL-2.0-only
//! Check every leaf codec first, then either move the whole refusal or keep the original Box.
use super::*;
use crate::sketch_region_prism::frozen::annular_history::AnnularHistoryFailure;

/// Project an actual annular publication refusal; `project_annular_publication_refusal(failure)`.
/// Projection coverage never changes the original phase or class.
pub(in crate::sketch_region_prism) fn project_annular_publication_refusal<'a>(
    original: Box<AnnularPublicationFailure<'a>>,
) -> AnnularPublicationRefusalProjection<'a> {
    let Some(payload) = pending(&original.cause) else {
        return moved(*original);
    };
    let diagnostic = Box::new(OwnedAnnularPublicationDiagnostic {
        operation: original.operation,
        requested: original.requested,
        phase: original.phase,
        class: original.class,
        cause: summary(&original.cause),
        gaps: vec![PublicationDiagnosticGap::Payload(payload)],
    });
    AnnularPublicationRefusalProjection {
        diagnostic,
        original: Some(original),
    }
}
/// The first leaf family without an explicit owned codec, if any.
fn pending(cause: &AnnularPublicationCause<'_>) -> Option<PublicationPendingPayload> {
    use AnnularPublicationCause as C;
    use PublicationPendingPayload as P;
    match cause {
        C::UnsupportedVersion { .. } | C::HistoryCancelled { .. } | C::Binding { .. } => None,
        C::Callback { downstream, .. } => downstream.as_deref().and_then(pending),
        C::Join(failure) => own_join(failure).err(),
        C::Validation(f) => annular_validation_failure_bytes(f)
            .is_none()
            .then_some(P::Validation),
        C::Measures(f) => annular_measure_failure_bytes(f)
            .is_none()
            .then_some(P::Measures),
        C::History(failure) => history_bytes(failure).is_none().then_some(P::History),
    }
}
/// Only a history refusal with no local owner and no nested join refusal moves.
fn history_bytes(failure: &AnnularHistoryFailure<'_>) -> Option<Vec<u8>> {
    let local = failure.join.is_some() || failure.history.local_diagnostic.is_some();
    (!local)
        .then(|| finding_bytes(&failure.history.finding).ok())
        .flatten()
}
fn moved(original: AnnularPublicationFailure<'_>) -> AnnularPublicationRefusalProjection<'_> {
    let diagnostic = Box::new(OwnedAnnularPublicationDiagnostic {
        operation: original.operation,
        requested: original.requested,
        phase: original.phase,
        class: original.class,
        cause: own(original.cause),
        gaps: Vec::new(),
    });
    AnnularPublicationRefusalProjection {
        diagnostic,
        original: None,
    }
}
const CHECKED: &str = "leaf codec checked before moving";
fn own(cause: AnnularPublicationCause<'_>) -> OwnedAnnularPublicationCause {
    use AnnularPublicationCause as C;
    use OwnedAnnularPublicationCause as O;
    match cause {
        C::Join(failure) => O::Join(Box::new(own_join(&failure).expect(CHECKED))),
        C::Validation(failure) => {
            let encoded = annular_validation_failure_bytes(&failure).expect(CHECKED);
            O::Validation(Box::new(OwnedAnnularChildRefusal { failure, encoded }))
        }
        C::Measures(failure) => {
            let encoded = annular_measure_failure_bytes(&failure).expect(CHECKED);
            O::Measures(Box::new(OwnedAnnularChildRefusal { failure, encoded }))
        }
        C::History(failure) => {
            let encoded = history_bytes(&failure).expect(CHECKED);
            let finding = failure.history.finding;
            O::History(Box::new(OwnedPublicationHistory { finding, encoded }))
        }
        C::Callback { stop, downstream } => O::Callback {
            stop: owned_callback(&stop),
            downstream: downstream.map(|child| Box::new(own(*child))),
        },
        other => scalar(&other),
    }
}
/// Retained-original summary: owned scalars and callbacks, the pending leaf as its family.
fn summary(cause: &AnnularPublicationCause<'_>) -> OwnedAnnularPublicationCause {
    use AnnularPublicationCause as C;
    use OwnedAnnularPublicationCause as O;
    match cause {
        C::Callback { stop, downstream } => O::Callback {
            stop: owned_callback(stop),
            downstream: downstream.as_deref().map(|child| Box::new(summary(child))),
        },
        C::UnsupportedVersion { .. } | C::HistoryCancelled { .. } | C::Binding { .. } => {
            scalar(cause)
        }
        other => O::Unprojected(pending(other).expect("summary of a pending leaf")),
    }
}
fn scalar(cause: &AnnularPublicationCause<'_>) -> OwnedAnnularPublicationCause {
    use AnnularPublicationCause as C;
    use OwnedAnnularPublicationCause as O;
    match cause {
        C::UnsupportedVersion { actual, expected } => O::UnsupportedVersion {
            actual: *actual,
            expected: *expected,
        },
        C::HistoryCancelled { operation, phase } => O::HistoryCancelled {
            operation: *operation,
            phase: *phase,
        },
        C::Binding { proof } => O::Binding { proof },
        _ => unreachable!("scalar annular publication causes only"),
    }
}
