// SPDX-License-Identifier: GPL-2.0-only
//! Borrow operation-local inputs; all persistent names are supplied before lookup.
use super::{progress::*, stage_error::*};
use crate::sketch_region::ArrangementSource;
use crate::sketch_region_contact_transport::PairedContactTransaction;
use crate::sketch_region_prism::{
    PrismReceipt,
    frozen::{PrismPublicationBindings, proof_join::*},
};

/// Live inputs stay operation-local; `source.compose(options, callbacks)` owns only the resulting join.
pub(crate) struct PrismPublicationSource<'a, 's> {
    pub bridge: &'a PairedContactTransaction,
    pub sources: &'a [ArrangementSource<'s>],
    pub receipt: &'a PrismReceipt,
    pub bindings: &'a PrismPublicationBindings,
    pub contact: &'a AuthoredClosureContact,
}
impl PrismPublicationSource<'_, '_> {
    pub(super) fn compose(
        &self,
        requested: PrismProofJoinOptions,
        callbacks: &mut PublicationCallbacks<'_>,
    ) -> Result<JoinedPrismProof, PublicationStageFailure> {
        // The existing join takes raw probes before invoking this first user callback.
        let proof = compose_prism_proof(
            self.bridge,
            self.sources,
            self.receipt,
            self.bindings,
            self.contact,
            requested,
            || callbacks.cancelled(PrismPublicationPhase::Join),
        );
        let proof = proof.map_err(|cause| join_failure(cause, PrismPublicationPhase::Join));
        finish_child(proof, callbacks)
    }
    pub(super) fn rebind(&self, join: &JoinedPrismProof) -> Result<(), PublicationStageFailure> {
        rebind_joined_prism_proof(self.bridge, self.sources, self.receipt, self.bindings, join)
            .map_err(|cause| join_failure(cause, PrismPublicationPhase::Commit))
    }
}
fn join_failure(
    cause: PrismProofJoinFailure,
    phase: PrismPublicationPhase,
) -> PublicationStageFailure {
    let class = match cause.cause {
        PrismProofJoinCause::FailedCapture(_) => PrismPublicationClass::Failed,
        PrismProofJoinCause::Cancelled { .. } => PrismPublicationClass::Cancelled,
        _ => PrismPublicationClass::Incomplete,
    };
    PublicationStageFailure {
        phase,
        class,
        cause: PublicationStageCause::Join(Box::new(cause)),
    }
}

#[cfg(test)]
#[path = "source_tests.rs"]
mod tests;
