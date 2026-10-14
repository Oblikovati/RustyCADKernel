// SPDX-License-Identifier: GPL-2.0-only
//! Pure child checks share one joined source; none of these helpers can publish a body.
use super::{progress::*, stage_error::*};
use crate::sketch_region_prism::frozen::{
    joined_measures::*, ordered_validation::*, proof_join::JoinedPrismProof,
};
use rustycad_tkernel::standard::Failure;

pub(super) fn validation(
    join: &JoinedPrismProof,
    requested: OrderedPrismValidationOptions,
    callbacks: &mut PublicationCallbacks<'_>,
) -> Result<Box<OrderedPrismValidationCertificate>, PublicationStageFailure> {
    let outcome = validate_joined_prism(
        join,
        requested,
        &mut PublicationValidationAdapter(callbacks),
    );
    let proof = match outcome {
        OrderedPrismValidationOutcome::CompleteValidation(proof) => Ok(proof),
        OrderedPrismValidationOutcome::Failed(cause)
        | OrderedPrismValidationOutcome::Incomplete(cause)
        | OrderedPrismValidationOutcome::Cancelled(cause) => Err(cause.into()),
    };
    let proof = proof.and_then(|proof| bound_validation(join, proof));
    finish_child(proof, callbacks)
}
fn bound_validation(
    join: &JoinedPrismProof,
    proof: Box<OrderedPrismValidationCertificate>,
) -> Result<Box<OrderedPrismValidationCertificate>, PublicationStageFailure> {
    if !proof.binds(join) {
        return Err(binding_failure(
            PrismPublicationPhase::Validation,
            "ordered validation",
        ));
    }
    Ok(proof)
}
pub(super) fn measures(
    join: &JoinedPrismProof,
    requested: JoinedPrismMeasureOptions,
    callbacks: &mut PublicationCallbacks<'_>,
) -> Result<JoinedPrismMeasures, PublicationStageFailure> {
    let proof = inspect_joined_prism_measures(join, requested, || {
        callbacks.cancelled(PrismPublicationPhase::Measures)
    })
    .map_err(PublicationStageFailure::from);
    let proof = proof.and_then(|proof| bound_measures(join, proof));
    finish_child(proof, callbacks)
}
fn bound_measures(
    join: &JoinedPrismProof,
    proof: JoinedPrismMeasures,
) -> Result<JoinedPrismMeasures, PublicationStageFailure> {
    if !proof.matches_join(join) {
        return Err(binding_failure(
            PrismPublicationPhase::Measures,
            "joined measures",
        ));
    }
    Ok(proof)
}
struct PublicationValidationAdapter<'a, 'b>(&'a mut PublicationCallbacks<'b>);
impl OrderedPrismValidationProgress for PublicationValidationAdapter<'_, '_> {
    fn checkpoint(&mut self, _: OrderedPrismCheckpoint) -> Result<bool, Failure> {
        Ok(self.0.cancelled(PrismPublicationPhase::Validation))
    }
}

#[cfg(test)]
#[path = "stage_tests.rs"]
mod tests;
