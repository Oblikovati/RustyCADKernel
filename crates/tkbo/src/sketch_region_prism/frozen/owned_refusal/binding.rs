// SPDX-License-Identifier: GPL-2.0-only
//! Completed projections never erase legacy classification, selection, or shape-bearing evidence.
use super::*;
pub(super) fn project(
    cause: PrismBindingFailure,
) -> ProjectionCheck<OwnedPrismBindingCause, PrismBindingFailure> {
    use OwnedPrismBindingCause as O;
    use PrismBindingFailure as L;
    match cause {
        L::Region(value) => region::project(value)
            .map(O::Region)
            .map_err(|e| e.map(PrismRefusalPath::Binding, L::Region)),
        L::Chart(value) => Ok(O::Chart(value)),
        L::Body(value) => body(value)
            .map(O::Body)
            .map_err(|e| e.map(PrismRefusalPath::Binding, L::Body)),
        L::Cancelled => Ok(O::Cancelled),
        L::Classification(value) => classification(*value),
    }
}
fn body(cause: PrismCompletion) -> ProjectionCheck<OwnedPrismBodyCause, PrismCompletion> {
    use PrismCompletion as L;
    match cause {
        L::Incomplete => Ok(OwnedPrismBodyCause::Incomplete),
        L::Cancelled(phase) => Ok(OwnedPrismBodyCause::Cancelled(phase)),
        L::Failed(cause) => sweep(cause)
            .map(OwnedPrismBodyCause::Failed)
            .map_err(|e| e.map(PrismRefusalPath::Body, L::Failed)),
        L::NoBody(selection) => Err(RetainedProjection::pending(
            L::NoBody(selection),
            PrismRefusalPath::Body,
            PendingPrismRefusalPayload::BodySelection,
        )),
    }
}
fn sweep(cause: PrismCause) -> ProjectionCheck<OwnedPrismSweepCause, PrismCause> {
    use OwnedPrismSweepCause as O;
    use PrismCause as L;
    match cause {
        L::Mapped { phase, failure } => Ok(O::Mapped { phase, failure }),
        L::ValidationRejected(stage) => Ok(O::ValidationRejected(stage)),
        L::InvalidRequest { quantity, actual } => Ok(O::InvalidRequest { quantity, actual }),
        L::Classification(value) => candidate::project(*value)
            .map(|v| O::Classification(Box::new(v)))
            .map_err(|e| e.map(PrismRefusalPath::Body, |v| L::Classification(Box::new(v)))),
        L::ClassificationProgress { failure, query } => classification_progress(failure, *query),
        L::CyclicBodyContext { .. } | L::MissingCopy { .. } => {
            pending_sweep(cause, PendingPrismRefusalPayload::BodyShape)
        }
    }
}
fn classification_progress(
    failure: Box<Failure>,
    query: crate::sketch_region_candidates::CandidateCause,
) -> ProjectionCheck<OwnedPrismSweepCause, PrismCause> {
    match candidate::project(query) {
        Ok(query) => Ok(OwnedPrismSweepCause::ClassificationProgress {
            failure,
            query: Box::new(query),
        }),
        Err(error) => Err(error.map(PrismRefusalPath::Body, |query| {
            PrismCause::ClassificationProgress {
                failure,
                query: Box::new(query),
            }
        })),
    }
}
fn pending_sweep(
    cause: PrismCause,
    payload: PendingPrismRefusalPayload,
) -> ProjectionCheck<OwnedPrismSweepCause, PrismCause> {
    let retained = RetainedProjection::pending(cause, PrismRefusalPath::Body, payload);
    Err(retained)
}

fn classification(
    value: crate::sketch_region_candidates::CandidateCause,
) -> ProjectionCheck<OwnedPrismBindingCause, PrismBindingFailure> {
    candidate::project(value)
        .map(|v| OwnedPrismBindingCause::Classification(Box::new(v)))
        .map_err(|e| {
            e.map(PrismRefusalPath::Binding, |v| {
                PrismBindingFailure::Classification(Box::new(v))
            })
        })
}
