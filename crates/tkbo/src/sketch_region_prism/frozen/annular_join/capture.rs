// SPDX-License-Identifier: GPL-2.0-only
use super::*;

pub(super) fn project(
    receipt: &PrismReceipt,
    names: &PrismPublicationBindings,
) -> AnnularFrozenCheck<'static, FrozenPrismProjection> {
    match super::super::freeze_prism_with_behavior(
        receipt,
        names,
        &mut CallbackFreeAnnularCapture,
        PrismFreezeBehavior::OrderedTraversalV2,
    ) {
        PrismFreezeOutcome::Captured(projection) => Ok(*projection),
        PrismFreezeOutcome::Incomplete(cause) => {
            Err(AnnularFrozenCause::IncompleteCapture(Box::new(cause)))
        }
        PrismFreezeOutcome::Failed(cause) => {
            Err(AnnularFrozenCause::FailedCapture(Box::new(cause)))
        }
        PrismFreezeOutcome::Cancelled { phase, .. } => Err(AnnularFrozenCause::Cancelled {
            freeze_phase: Some(phase),
        }),
    }
}
pub(super) struct AnnularJoinedOperands {
    pub faces: Vec<NamedAnnularFace>,
    pub parents: Vec<JoinedParentDispositions>,
    pub raw_vertices: Vec<JoinedVertexOperand>,
    pub replay: Vec<u8>,
}
impl<'r> operation::AnnularJoinInputs<'_, 'r, '_> {
    pub(super) fn operands(
        &self,
        proof: &AnnularEmbeddingEvidence,
        projection: &FrozenPrismProjection,
        census: AnnularNameCensus,
    ) -> AnnularFrozenCheck<'r, AnnularJoinedOperands> {
        let bound = bind_projection(self.receipt, self.authored)?;
        let mut result = AnnularJoinedOperands {
            faces: links::capture(proof, &bound, projection)?,
            parents: link_frozen_parent_dispositions(&bound, projection)?,
            raw_vertices: capture_frozen_raw_vertices(&bound, projection)?,
            replay: Vec::new(),
        };
        result.replay = replay::encode(proof, projection, census, &result, &bound)?;
        Ok(result)
    }
}

// Native capture follows the bounded current-binding check with no external callback gap.
struct CallbackFreeAnnularCapture;
impl FreezeProgress for CallbackFreeAnnularCapture {
    fn checkpoint(&mut self, _: FreezePhase) -> Result<bool, rustycad_tkernel::standard::Failure> {
        Ok(false)
    }
}
