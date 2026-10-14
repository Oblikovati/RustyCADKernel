// SPDX-License-Identifier: GPL-2.0-only
//! Child stages share one sealed join and one callback sequence; none publishes a body.
use super::*;
use rustycad_tkernel::standard::Failure;

/// One stage refusal before the operation identity is attached.
pub(super) struct AnnularStageStop<'a> {
    pub phase: PrismPublicationPhase,
    pub class: PrismPublicationClass,
    pub cause: AnnularPublicationCause<'a>,
}
pub(super) type StageCheck<'a, T> = Result<T, AnnularStageStop<'a>>;
/// Join the sealed embedding to named frozen capture under `Join` callbacks.
pub(super) fn join<'a>(
    request: &AnnularPublicationRequest<'a, '_, '_>,
    embedding: AnnularEmbeddingEvidence,
    callbacks: &mut PublicationCallbacks<'_>,
) -> StageCheck<'a, AnnularFrozenDomain> {
    let (region, receipt, names) = (request.region, request.receipt, request.bindings);
    let options = request.requested.join;
    let mut adapter = JoinAdapter(callbacks);
    let joined =
        join_annular_frozen_domain(region, receipt, names, embedding, options, &mut adapter);
    let joined = joined.map_err(|failure| join_stop(failure, PrismPublicationPhase::Join));
    finish(joined, callbacks)
}
/// Classify one annular join refusal without discarding its carrier.
pub(super) fn join_stop(
    failure: Box<AnnularFrozenFailure<'_>>,
    phase: PrismPublicationPhase,
) -> AnnularStageStop<'_> {
    let class = match failure.classification() {
        TwoLoopRefusalClass::Failed => PrismPublicationClass::Failed,
        TwoLoopRefusalClass::Cancelled => PrismPublicationClass::Cancelled,
        TwoLoopRefusalClass::Incomplete => PrismPublicationClass::Incomplete,
    };
    let cause = AnnularPublicationCause::Join(failure);
    AnnularStageStop {
        phase,
        class,
        cause,
    }
}
/// Ordered exact validation over the shared join.
pub(super) fn validation<'a>(
    join: &AnnularFrozenDomain,
    requested: AnnularValidationOptions,
    callbacks: &mut PublicationCallbacks<'_>,
) -> StageCheck<'a, AnnularValidationCertificate> {
    let outcome = validate_annular_prism(join, requested, &mut ValidationAdapter(callbacks));
    let proof = outcome.map_err(validation_stop);
    let phase = PrismPublicationPhase::Validation;
    let proof = proof.and_then(|p| bound(p.binds(join), phase, "ordered validation").map(|_| p));
    finish(proof, callbacks)
}
fn validation_stop<'a>(failure: Box<AnnularValidationFailure>) -> AnnularStageStop<'a> {
    let class = match failure.class {
        OrderedPrismFailureClass::Failed => PrismPublicationClass::Failed,
        OrderedPrismFailureClass::Incomplete => PrismPublicationClass::Incomplete,
        OrderedPrismFailureClass::Cancelled => PrismPublicationClass::Cancelled,
    };
    let phase = PrismPublicationPhase::Validation;
    let cause = AnnularPublicationCause::Validation(failure);
    AnnularStageStop {
        phase,
        class,
        cause,
    }
}
/// Conditional four-face measures over the shared join.
pub(super) fn measures<'a>(
    join: &AnnularFrozenDomain,
    requested: AnnularMeasureOptions,
    callbacks: &mut PublicationCallbacks<'_>,
) -> StageCheck<'a, AnnularPrismMeasures> {
    let phase = PrismPublicationPhase::Measures;
    let proof = inspect_annular_measures(join, requested, || callbacks.cancelled(phase));
    let proof = proof.map_err(measure_stop);
    let proof = proof.and_then(|p| bound(p.matches(join), phase, "annular measures").map(|_| p));
    finish(proof, callbacks)
}
fn measure_stop<'a>(failure: Box<AnnularMeasureFailure>) -> AnnularStageStop<'a> {
    let class = match failure.classification() {
        JoinedMeasureFailureClass::Incomplete => PrismPublicationClass::Incomplete,
        JoinedMeasureFailureClass::Cancelled => PrismPublicationClass::Cancelled,
    };
    let phase = PrismPublicationPhase::Measures;
    let cause = AnnularPublicationCause::Measures(failure);
    AnnularStageStop {
        phase,
        class,
        cause,
    }
}
/// A child result that does not bind the shared join is incomplete, never accepted.
pub(super) fn bound<'a>(
    binds: bool,
    phase: PrismPublicationPhase,
    proof: &'static str,
) -> StageCheck<'a, ()> {
    if binds {
        return Ok(());
    }
    let (class, cause) = (
        PrismPublicationClass::Incomplete,
        AnnularPublicationCause::Binding { proof },
    );
    Err(AnnularStageStop {
        phase,
        class,
        cause,
    })
}
/// Attach a recorded callback stop to the child result it interrupted.
pub(super) fn finish<'a, T>(
    result: StageCheck<'a, T>,
    callbacks: &mut PublicationCallbacks<'_>,
) -> StageCheck<'a, T> {
    let Some(stop) = callbacks.take_stop() else {
        return result;
    };
    let class = match stop.cause {
        PublicationCallbackCause::Cancelled => PrismPublicationClass::Cancelled,
        PublicationCallbackCause::Failed(_) => PrismPublicationClass::Failed,
        PublicationCallbackCause::Exhausted => PrismPublicationClass::Incomplete,
    };
    let phase = stop.checkpoint.phase;
    let downstream = result.err().map(|child| Box::new(child.cause));
    let cause = AnnularPublicationCause::Callback {
        stop: Box::new(stop),
        downstream,
    };
    Err(AnnularStageStop {
        phase,
        class,
        cause,
    })
}
struct JoinAdapter<'a, 'b>(&'a mut PublicationCallbacks<'b>);
impl AnnularFrozenProgress for JoinAdapter<'_, '_> {
    fn checkpoint(&mut self, _: AnnularFrozenPhase) -> Result<bool, Failure> {
        Ok(self.0.cancelled(PrismPublicationPhase::Join))
    }
}
struct ValidationAdapter<'a, 'b>(&'a mut PublicationCallbacks<'b>);
impl AnnularValidationProgress for ValidationAdapter<'_, '_> {
    fn checkpoint(&mut self, _: AnnularValidationCheckpoint) -> Result<bool, Failure> {
        Ok(self.0.cancelled(PrismPublicationPhase::Validation))
    }
}
