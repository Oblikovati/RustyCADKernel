// SPDX-License-Identifier: GPL-2.0-only
//! Preserve child proof refusals and the initiating callback cause together.
use super::progress::*;
use crate::sketch_region_prism::frozen::{joined_measures::*, ordered_validation::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismPublicationClass {
    Failed,
    Incomplete,
    Cancelled,
}
pub(super) struct PublicationStageFailure {
    pub phase: PrismPublicationPhase,
    pub class: PrismPublicationClass,
    pub cause: PublicationStageCause,
}
pub(super) enum PublicationStageCause {
    Validation(Box<OrderedPrismValidationFailure>),
    Measures(Box<JoinedPrismMeasureFailure>),
    Binding {
        proof: &'static str,
    },
    Callback {
        stop: PublicationCallbackStop,
        downstream: Option<Box<PublicationStageCause>>,
    },
}
pub(super) fn binding_failure(
    phase: PrismPublicationPhase,
    proof: &'static str,
) -> PublicationStageFailure {
    PublicationStageFailure {
        phase,
        class: PrismPublicationClass::Incomplete,
        cause: PublicationStageCause::Binding { proof },
    }
}
pub(super) fn finish_child<T>(
    result: Result<T, PublicationStageFailure>,
    callbacks: &mut PublicationCallbacks<'_>,
) -> Result<T, PublicationStageFailure> {
    let Some(stop) = callbacks.take_stop() else {
        return result;
    };
    let phase = stop.checkpoint.phase;
    let class = match stop.cause {
        PublicationCallbackCause::Cancelled => PrismPublicationClass::Cancelled,
        PublicationCallbackCause::Failed(_) => PrismPublicationClass::Failed,
        PublicationCallbackCause::Exhausted => PrismPublicationClass::Incomplete,
    };
    let downstream = result.err().map(|failure| Box::new(failure.cause));
    Err(PublicationStageFailure {
        phase,
        class,
        cause: PublicationStageCause::Callback { stop, downstream },
    })
}
impl From<Box<OrderedPrismValidationFailure>> for PublicationStageFailure {
    fn from(cause: Box<OrderedPrismValidationFailure>) -> Self {
        let class = match cause.class {
            OrderedPrismFailureClass::Failed => PrismPublicationClass::Failed,
            OrderedPrismFailureClass::Incomplete => PrismPublicationClass::Incomplete,
            OrderedPrismFailureClass::Cancelled => PrismPublicationClass::Cancelled,
        };
        Self {
            phase: PrismPublicationPhase::Validation,
            class,
            cause: PublicationStageCause::Validation(cause),
        }
    }
}
impl From<Box<JoinedPrismMeasureFailure>> for PublicationStageFailure {
    fn from(cause: Box<JoinedPrismMeasureFailure>) -> Self {
        let class = match cause.class {
            JoinedMeasureFailureClass::Incomplete => PrismPublicationClass::Incomplete,
            JoinedMeasureFailureClass::Cancelled => PrismPublicationClass::Cancelled,
        };
        Self {
            phase: PrismPublicationPhase::Measures,
            class,
            cause: PublicationStageCause::Measures(cause),
        }
    }
}
