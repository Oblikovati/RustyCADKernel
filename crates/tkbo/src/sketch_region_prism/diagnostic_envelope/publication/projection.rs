// SPDX-License-Identifier: GPL-2.0-only
//! Preserve actual headers; a V1 callback descendant has no recoverable child header.
use super::*;
use crate::sketch_region_prism::frozen::PrismPublicationBindings;
struct ProjectionContext<'a> {
    bindings: &'a PrismPublicationBindings,
    operation_class: PrismPublicationClass,
    attributed: bool,
    gaps: &'a mut Vec<PublicationDiagnosticGap>,
}
pub(super) fn outer(
    original: &PrismPublicationFailure<'_>,
    bindings: &PrismPublicationBindings,
    gaps: &mut Vec<PublicationDiagnosticGap>,
) -> OwnedPublicationDiagnosticCause {
    let attributed = gaps.is_empty();
    let mut context = ProjectionContext {
        bindings,
        operation_class: original.class,
        attributed,
        gaps,
    };
    context.outer_cause(&original.cause)
}
impl ProjectionContext<'_> {
    fn outer_cause(
        &mut self,
        value: &PrismPublicationCause<'_>,
    ) -> OwnedPublicationDiagnosticCause {
        use OwnedPublicationDiagnosticCause as O;
        match value {
            PrismPublicationCause::UnsupportedVersion { actual, expected } => {
                O::UnsupportedVersion {
                    actual: *actual,
                    expected: *expected,
                }
            }
            PrismPublicationCause::Stage(value) => O::Stage(Box::new(self.stage(value))),
            PrismPublicationCause::HistoryCancelled { operation, phase } => {
                self.history_cancelled(*operation, *phase)
            }
            PrismPublicationCause::History(_) => self.pending_history(),
        }
    }
    fn history_cancelled(
        &mut self,
        operation: AuthoredPrismId,
        phase: PrismHistoryPhase,
    ) -> OwnedPublicationDiagnosticCause {
        if operation != self.bindings.operation {
            self.gaps.push(PublicationDiagnosticGap::Operation {
                actual: operation,
                expected: self.bindings.operation,
            });
        }
        OwnedPublicationDiagnosticCause::HistoryCancelled { operation, phase }
    }
    fn pending_history(&mut self) -> OwnedPublicationDiagnosticCause {
        self.pending(PublicationPendingPayload::History);
        OwnedPublicationDiagnosticCause::Unprojected(PublicationPendingPayload::History)
    }
    fn stage(&mut self, value: &PublicationStageFailure) -> OwnedPublicationStageDiagnostic {
        let origin = PublicationClassOrigin::RetainedStage(value.phase);
        let cause = self.cause(&value.cause, value.class, origin);
        OwnedPublicationStageDiagnostic {
            phase: value.phase,
            class: value.class,
            cause,
        }
    }
    fn cause(
        &mut self,
        value: &PublicationStageCause,
        class: PrismPublicationClass,
        origin: PublicationClassOrigin,
    ) -> OwnedPublicationStageCause {
        use PublicationStageCause as C;
        match value {
            C::Join(value) => self.join(value, class, origin),
            C::Binding { proof } => OwnedPublicationStageCause::Binding { proof },
            C::Validation(_) => self.pending(PublicationPendingPayload::Validation),
            C::Measures(_) => self.pending(PublicationPendingPayload::Measures),
            C::Callback { stop, downstream } => self.legacy_callback(stop, downstream.as_deref()),
            C::CallbackWithChild { stop, downstream } => {
                self.retained_callback(stop, downstream.as_deref())
            }
        }
    }
    fn retained_callback(
        &mut self,
        stop: &PublicationCallbackStop,
        downstream: Option<&PublicationStageFailure>,
    ) -> OwnedPublicationStageCause {
        OwnedPublicationStageCause::CallbackV2 {
            stop: callback(stop),
            downstream: downstream.map(|value| Box::new(self.stage(value))),
        }
    }
    fn legacy_callback(
        &mut self,
        stop: &PublicationCallbackStop,
        downstream: Option<&PublicationStageCause>,
    ) -> OwnedPublicationStageCause {
        let downstream = downstream.map(|value| {
            self.gaps
                .push(PublicationDiagnosticGap::UnretainedChildHeader);
            Box::new(self.cause(
                value,
                self.operation_class,
                PublicationClassOrigin::PublicationOperation,
            ))
        });
        OwnedPublicationStageCause::CallbackV1 {
            stop: callback(stop),
            downstream,
        }
    }
    fn join(
        &mut self,
        value: &PrismProofJoinFailure,
        class: PrismPublicationClass,
        origin: PublicationClassOrigin,
    ) -> OwnedPublicationStageCause {
        if !self.attributed || !self.join_identity(value) {
            return self.pending(joined::family(&value.cause));
        }
        match joined::project(value, class, self.bindings) {
            Ok((evidence, encoded)) => OwnedPublicationStageCause::Join {
                evidence,
                encoded,
                class_origin: origin,
            },
            Err(payload) => self.pending(payload),
        }
    }
    fn join_identity(&mut self, value: &PrismProofJoinFailure) -> bool {
        if value.operation != self.bindings.operation {
            self.gaps.push(PublicationDiagnosticGap::Operation {
                actual: value.operation,
                expected: self.bindings.operation,
            });
            return false;
        }
        if let Some(entity) = value.entity {
            self.gaps
                .push(PublicationDiagnosticGap::UnprojectedEntity(entity));
            return false;
        }
        true
    }
    fn pending(&mut self, payload: PublicationPendingPayload) -> OwnedPublicationStageCause {
        self.gaps.push(PublicationDiagnosticGap::Payload(payload));
        OwnedPublicationStageCause::Unprojected(payload)
    }
}
fn callback(value: &PublicationCallbackStop) -> OwnedPublicationCallback {
    let cause = match &value.cause {
        PublicationCallbackCause::Cancelled => OwnedDiagnosticCallbackCause::Cancelled,
        PublicationCallbackCause::Failed(value) => {
            OwnedDiagnosticCallbackCause::Failed(value.clone())
        }
        PublicationCallbackCause::Exhausted => OwnedDiagnosticCallbackCause::Exhausted,
    };
    OwnedPublicationCallback {
        checkpoint: value.checkpoint,
        cause,
    }
}
