// SPDX-License-Identifier: GPL-2.0-only
//! Primitive projection fixtures do not issue packets or qualify detached contexts.
mod admission;
mod joins;
mod stages;
use super::*;
use crate::sketch_region_prism::frozen::PrismPublicationBindings;
use crate::sketch_region_prism::owned_request::publication_diagnostic_tests::with_diagnostic_context;
static_assertions::assert_impl_all!(OwnedPrismPublicationDiagnostic: Send, Sync);
static_assertions::assert_impl_all!(OwnedPublicationDiagnosticCause: Send, Sync);
static_assertions::assert_impl_all!(OwnedPublicationStageCause: Send, Sync);
static_assertions::assert_impl_all!(OwnedPublicationStageDiagnostic: Send, Sync);
static_assertions::assert_impl_all!(OwnedPublicationCallback: Send, Sync);
static_assertions::assert_impl_all!(PublicationDiagnosticGap: Send, Sync);
static_assertions::assert_impl_all!(PublicationPendingPayload: Send, Sync);
static_assertions::assert_impl_all!(PublicationClassOrigin: Send, Sync);
fn join(cause: PrismProofJoinCause, bindings: &PrismPublicationBindings) -> PrismProofJoinFailure {
    PrismProofJoinFailure {
        operation: bindings.operation,
        phase: PrismProofJoinPhase::Admission,
        entity: None,
        cause,
    }
}
fn outer(
    cause: PublicationStageCause,
    options: PrismPublicationOptions,
) -> PrismPublicationFailure<'static> {
    PrismPublicationFailure {
        operation: AuthoredPrismId(257),
        requested: options,
        phase: PrismPublicationPhase::Join,
        class: PrismPublicationClass::Failed,
        cause: PrismPublicationCause::Stage(Box::new(PublicationStageFailure {
            phase: PrismPublicationPhase::Join,
            class: PrismPublicationClass::Failed,
            cause,
        })),
    }
}
fn stage(value: &OwnedPublicationDiagnosticCause) -> &OwnedPublicationStageDiagnostic {
    let OwnedPublicationDiagnosticCause::Stage(value) = value else {
        panic!("expected stage")
    };
    value
}
