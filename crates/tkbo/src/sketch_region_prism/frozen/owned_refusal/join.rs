// SPDX-License-Identifier: GPL-2.0-only
//! Projection completeness never changes the originating operation's supplied failure class.
use super::*;
/// Preserve the local failure when a V1 projection is incomplete; `project_join_refusal(error,class,&context)`.
pub(crate) fn project_join_refusal(
    failure: PrismProofJoinFailure,
    class: PrismRefusalClass,
    context: &PrismRefusalIdentityContext<'_>,
) -> PrismJoinRefusalProjection {
    if failure.operation.0 == 0 {
        let cause = PrismRefusalProjectionGap::InvalidOperation(failure.operation);
        return incomplete(failure, class, join_issue(cause));
    }
    if failure.operation != context.bindings.operation {
        let issue = operation_issue(failure.operation, context.bindings.operation);
        return incomplete(failure, class, issue);
    }
    project_bound(failure, class, context)
}
fn operation_issue(
    expected: AuthoredPrismId,
    actual: AuthoredPrismId,
) -> PrismRefusalProjectionIssue {
    let cause = PrismRefusalProjectionGap::Operation { actual, expected };
    join_issue(cause)
}
fn join_issue(cause: PrismRefusalProjectionGap) -> PrismRefusalProjectionIssue {
    PrismRefusalProjectionIssue {
        path: vec![PrismRefusalPath::Join],
        cause,
    }
}
fn project_bound(
    failure: PrismProofJoinFailure,
    class: PrismRefusalClass,
    context: &PrismRefusalIdentityContext<'_>,
) -> PrismJoinRefusalProjection {
    let (header, cause) = JoinHeader::from_local(failure, class);
    match project_cause(cause, context) {
        Ok(cause) => header.complete(cause),
        Err(error) => header.incomplete(*error),
    }
}
struct JoinHeader {
    operation: AuthoredPrismId,
    phase: PrismProofJoinPhase,
    entity: Option<FrozenPrismKey>,
    class: PrismRefusalClass,
}
impl JoinHeader {
    fn from_local(
        failure: PrismProofJoinFailure,
        class: PrismRefusalClass,
    ) -> (Self, PrismProofJoinCause) {
        let header = Self {
            operation: failure.operation,
            phase: failure.phase,
            entity: failure.entity,
            class,
        };
        (header, failure.cause)
    }

    fn complete(self, cause: OwnedPrismJoinCause) -> PrismJoinRefusalProjection {
        let refusal = OwnedPrismJoinRefusal {
            operation: self.operation,
            phase: self.phase,
            entity: self.entity,
            class: self.class,
            cause,
        };
        PrismJoinRefusalProjection::Projected(Box::new(refusal))
    }
    fn incomplete(
        self,
        error: RetainedProjection<PrismProofJoinCause>,
    ) -> PrismJoinRefusalProjection {
        let failure = PrismProofJoinFailure {
            operation: self.operation,
            phase: self.phase,
            entity: self.entity,
            cause: error.original,
        };
        incomplete(failure, self.class, error.issue)
    }
}
fn incomplete(
    original: PrismProofJoinFailure,
    class: PrismRefusalClass,
    issue: PrismRefusalProjectionIssue,
) -> PrismJoinRefusalProjection {
    let failure = LocalPrismProjectionFailure {
        class,
        issue,
        original,
    };
    PrismJoinRefusalProjection::Incomplete(Box::new(failure))
}
fn project_cause(
    cause: PrismProofJoinCause,
    context: &PrismRefusalIdentityContext<'_>,
) -> ProjectionCheck<OwnedPrismJoinCause, PrismProofJoinCause> {
    use OwnedPrismJoinCause as O;
    use PrismProofJoinCause as L;
    match cause {
        L::MissingClosureQuery => Ok(O::MissingClosureQuery),
        L::Version { actual, expected } => Ok(O::Version { actual, expected }),
        L::IncompleteCapture(value) => project_incomplete_capture(*value),
        L::FailedCapture(value) => project_failed_capture(*value),
        L::Link(value) => map_join(frozen_diagnostic::project_link(*value), O::Link, L::Link),
        L::Cancelled { freeze_phase } => Ok(O::Cancelled { freeze_phase }),
        L::Stale { representation } => Ok(O::Stale { representation }),
        L::ClosureQuery(value) => project_source(*value, context),
        L::ClosureRegion(value) => project_region(*value),
        L::Embedding(value) => project_embedding(*value, context),
        L::Binding(value) => map_join(binding::project(*value), O::Binding, L::Binding),
    }
}
fn map_join<T, L>(
    result: ProjectionCheck<T, L>,
    owned: impl FnOnce(Box<T>) -> OwnedPrismJoinCause,
    local: impl FnOnce(Box<L>) -> PrismProofJoinCause,
) -> ProjectionCheck<OwnedPrismJoinCause, PrismProofJoinCause> {
    result
        .map(|value| owned(Box::new(value)))
        .map_err(|error| error.map(PrismRefusalPath::Join, |value| local(Box::new(value))))
}

fn project_source(
    value: SourceConformanceCause,
    context: &PrismRefusalIdentityContext<'_>,
) -> ProjectionCheck<OwnedPrismJoinCause, PrismProofJoinCause> {
    let result = source::project(value, context);
    map_join(
        result,
        OwnedPrismJoinCause::ClosureQuery,
        PrismProofJoinCause::ClosureQuery,
    )
}
fn project_region(
    value: RegionCertificateCause,
) -> ProjectionCheck<OwnedPrismJoinCause, PrismProofJoinCause> {
    let result = region::project(value);
    map_join(
        result,
        OwnedPrismJoinCause::ClosureRegion,
        PrismProofJoinCause::ClosureRegion,
    )
}
fn project_embedding(
    value: PrismEmbeddingFailure,
    context: &PrismRefusalIdentityContext<'_>,
) -> ProjectionCheck<OwnedPrismJoinCause, PrismProofJoinCause> {
    let result = embedding::project(value, context);
    map_join(
        result,
        OwnedPrismJoinCause::Embedding,
        PrismProofJoinCause::Embedding,
    )
}

fn project_incomplete_capture(
    value: PrismFreezeFailure,
) -> ProjectionCheck<OwnedPrismJoinCause, PrismProofJoinCause> {
    let result = frozen_diagnostic::project_capture(value);
    map_join(
        result,
        OwnedPrismJoinCause::IncompleteCapture,
        PrismProofJoinCause::IncompleteCapture,
    )
}

fn project_failed_capture(
    value: PrismFreezeFailure,
) -> ProjectionCheck<OwnedPrismJoinCause, PrismProofJoinCause> {
    let result = frozen_diagnostic::project_capture(value);
    map_join(
        result,
        OwnedPrismJoinCause::FailedCapture,
        PrismProofJoinCause::FailedCapture,
    )
}
