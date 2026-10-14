// SPDX-License-Identifier: GPL-2.0-only
//! Refusals stay operation-local and preserve original typed diagnostics.
use super::{progress::*, request::*, result::*, stage_error::*};
use crate::sketch_region_prism::frozen::complete_history::*;
impl<'a> PrismPublicationRequest<'a> {
    pub(super) fn failure(
        &self,
        phase: PrismPublicationPhase,
        class: PrismPublicationClass,
        cause: PrismPublicationCause<'a>,
    ) -> Box<PrismPublicationFailure<'a>> {
        Box::new(PrismPublicationFailure {
            operation: self.source.bindings.operation,
            requested: self.requested,
            phase,
            class,
            cause,
        })
    }
    pub(super) fn stage_failure(
        &self,
        failure: PublicationStageFailure,
    ) -> Box<PrismPublicationFailure<'a>> {
        self.failure(
            failure.phase,
            failure.class,
            PrismPublicationCause::Stage(Box::new(failure)),
        )
    }
    pub(super) fn history_failure(
        &self,
        failure: PrismHistoryFailure<'a>,
        phase: PrismPublicationPhase,
    ) -> Box<PrismPublicationFailure<'a>> {
        let class = match failure.finding.class {
            PrismHistoryFailureClass::Failed => PrismPublicationClass::Failed,
            PrismHistoryFailureClass::Incomplete => PrismPublicationClass::Incomplete,
        };
        self.failure(
            phase,
            class,
            PrismPublicationCause::History(Box::new(failure)),
        )
    }
    pub(super) fn callback(
        &self,
        phase: PrismPublicationPhase,
        callbacks: &mut PublicationCallbacks<'_>,
    ) -> PublicationCheck<'a, ()> {
        callbacks.cancelled(phase);
        finish_child(Ok(()), callbacks).map_err(|cause| self.stage_failure(cause))
    }
}
