// SPDX-License-Identifier: GPL-2.0-only
use super::*;

/// Rebind every retained operand after downstream callbacks; `rebind_annular_frozen_domain(request,receipt,names,joined)`.
/// This makes no callback, and cannot promote the conditional domain proof to a valid body.
pub(in crate::sketch_region_prism) fn rebind_annular_frozen_domain<'r>(
    request: &TwoLoopRegionRequest<'r, '_>,
    receipt: &PrismReceipt,
    authored: &PrismPublicationBindings,
    joined: &AnnularFrozenDomain,
) -> Result<(), Box<AnnularFrozenFailure<'r>>> {
    let inputs = operation::AnnularJoinInputs {
        request,
        receipt,
        authored,
    };
    inputs
        .rebind(joined)
        .map_err(|cause| inputs.failure(AnnularFrozenPhase::Commit, joined.census.requested, cause))
}
impl<'r> operation::AnnularJoinInputs<'_, 'r, '_> {
    pub(super) fn rebind(&self, joined: &AnnularFrozenDomain) -> AnnularFrozenCheck<'r, ()> {
        let census = census::inspect(self.authored, joined.census.requested)?;
        self.current_names(&joined.embedding)?;
        super::super::admission::admit_receipt(self.receipt)?;
        let bound = bind_projection(self.receipt, self.authored)?;
        super::super::capture::check_final_capture(
            &bound,
            &joined.projection,
            PrismFreezeBehavior::OrderedTraversalV2,
        )?;
        let actual = self.operands(&joined.embedding, &joined.projection, census)?;
        if actual.replay != joined.replay {
            return Err(PrismFreezeCause::StaleCapture {
                representation: "complete named annular domain operands and binding policy",
            }
            .into());
        }
        Ok(())
    }
}
