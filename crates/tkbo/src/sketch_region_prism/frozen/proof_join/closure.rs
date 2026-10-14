// SPDX-License-Identifier: GPL-2.0-only
//! The semantic closure census is bound before any raw operand enters persistent replay.
use super::*;
use crate::sketch_region::{ArrangementSource, PartitionOccurrenceIdentity};
use crate::sketch_region_contact_transport::{
    PairedContactTransaction, capture_contact_point_query,
};
use crate::sketch_region_prism::{
    PrismReceipt,
    region_certificate::{
        ClosurePointContext, ClosurePointStage, RegionCertificateCause, current_closure_vertices,
    },
};
use rustycad_tkbrep::topods::Shape;

pub(super) fn capture(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    proof: &PrismEmbeddingEvidence,
    bindings: &PrismPublicationBindings,
    contact: &AuthoredClosureContact,
) -> Result<JoinedClosureOperands, PrismProofJoinCause> {
    let actual = current_closure_vertices(bridge, sources, receipt).map_err(region_failure)?;
    let rows = capture_rows(&actual, proof, bindings, contact, bridge)?;
    let query = capture_contact_point_query(bridge, contact_index(bridge, contact)?)
        .ok_or(PrismProofJoinCause::MissingClosureQuery)?
        .map_err(|cause| PrismProofJoinCause::ClosureQuery(Box::new(cause)))?;
    if query.original != proof.binding().region.original {
        return Err(PrismProofJoinCause::Stale {
            representation: "original closure query parent",
        });
    }
    Ok(JoinedClosureOperands { rows, query })
}
fn region_failure(cause: RegionCertificateCause) -> PrismProofJoinCause {
    PrismProofJoinCause::ClosureRegion(Box::new(cause))
}
struct ClosureCensus<'a> {
    expected: Vec<&'a ClosurePointContext>,
    matched: Vec<bool>,
    original: FrozenOriginalKey,
    contact: AuthoredPrismId,
    selected: bool,
}
fn capture_rows(
    actual: &[(ClosurePointStage, Shape)],
    proof: &PrismEmbeddingEvidence,
    bindings: &PrismPublicationBindings,
    contact: &AuthoredClosureContact,
    bridge: &PairedContactTransaction,
) -> Result<Vec<JoinedClosurePointOperand>, PrismProofJoinCause> {
    let mut census = ClosureCensus::new(proof, bindings, contact, bridge)?;
    let rows = actual
        .iter()
        .map(|(stage, shape)| census.capture(*stage, shape))
        .collect::<Result<Vec<_>, _>>()?;
    check_complete(&census.matched, rows.len())?;
    Ok(rows)
}
impl<'a> ClosureCensus<'a> {
    fn new(
        proof: &'a PrismEmbeddingEvidence,
        bindings: &PrismPublicationBindings,
        contact: &AuthoredClosureContact,
        bridge: &PairedContactTransaction,
    ) -> Result<Self, PrismProofJoinCause> {
        let region = &proof.binding().region;
        let expected: Vec<_> = region
            .closure_points
            .iter()
            .flat_map(|row| &row.contexts)
            .collect();
        let matched = vec![false; expected.len()];
        let selected =
            replay_sources::first_is_selected(bridge, region).map_err(join_link_cause)?;
        let original = original_key(bindings, &region.original).map_err(join_link_cause)?;
        Ok(Self {
            expected,
            matched,
            original,
            contact: contact.name,
            selected,
        })
    }
    fn capture(
        &mut self,
        stage: ClosurePointStage,
        shape: &Shape,
    ) -> Result<JoinedClosurePointOperand, PrismProofJoinCause> {
        match_context(
            stage,
            &PartitionOccurrenceIdentity::capture(shape),
            &self.expected,
            &mut self.matched,
        )?;
        let local_point = raw_vertices::local_point(shape).map_err(join_link_cause)?;
        let placement =
            super::super::history::freeze_placement(shape.location()).map_err(join_link_cause)?;
        Ok(JoinedClosurePointOperand {
            contact: self.contact,
            original: self.original.clone(),
            stage: replay_sources::semantic_stage(stage, self.selected),
            orientation: shape.orientation(),
            local_point,
            placement,
        })
    }
}
fn match_context(
    stage: ClosurePointStage,
    actual: &PartitionOccurrenceIdentity,
    expected: &[&ClosurePointContext],
    matched: &mut [bool],
) -> Result<(), PrismProofJoinCause> {
    let matches: Vec<_> = expected
        .iter()
        .enumerate()
        .filter(|(_, row)| row.stage == stage && row.occurrence == *actual)
        .map(|(index, _)| index)
        .collect();
    let index = *only_bound(&matches, "one legacy semantic directed closure context")
        .map_err(join_link_cause)?;
    if matched[index] {
        return Err(join_link_cause(identity(
            "unique actual closure stage context",
            2,
        )));
    }
    matched[index] = true;
    Ok(())
}
fn check_complete(matched: &[bool], actual: usize) -> Result<(), PrismProofJoinCause> {
    if actual != 12 || matched.len() != 12 || matched.iter().any(|value| !value) {
        return Err(join_link_cause(identity(
            "bijection of all twelve semantic closure uses",
            actual,
        )));
    }
    Ok(())
}

fn contact_index(
    bridge: &PairedContactTransaction,
    contact: &AuthoredClosureContact,
) -> Result<usize, PrismProofJoinCause> {
    let matches: Vec<_> = bridge
        .inspection
        .contacts
        .iter()
        .enumerate()
        .filter(|(_, row)| row.identity == contact.actual)
        .map(|(index, _)| index)
        .collect();
    only_bound(&matches, "unique actual authored closure contact query")
        .copied()
        .map_err(join_link_cause)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketch_region_prism::region_certificate::{ClosurePointContext, ClosurePointStage};
    use rustycad_tkbrep::topods::Shape;
    #[test]
    fn closure_context_bijection_refuses_duplicates_missing_and_wrong_stage() {
        let (_, shapes) = crate::sketch_region_prism::region_certificate::tests::fixture();
        let key = PartitionOccurrenceIdentity::capture(&shapes[0]);
        let context = ClosurePointContext {
            stage: ClosurePointStage::Original,
            occurrence: key.clone(),
        };
        let mut matched = [false];
        assert!(
            match_context(ClosurePointStage::Original, &key, &[&context], &mut matched).is_ok()
        );
        assert!(
            match_context(ClosurePointStage::Original, &key, &[&context], &mut matched).is_err()
        );
        assert!(match_context(ClosurePointStage::Census, &key, &[&context], &mut [false]).is_err());
        assert!(
            match_context(
                ClosurePointStage::Original,
                &key,
                &[&context, &context],
                &mut [false; 2]
            )
            .is_err()
        );
        assert!(check_complete(&[true; 11], 11).is_err());
        assert!(check_complete(&[false; 12], 12).is_err());
        assert!(check_complete(&[true; 12], 12).is_ok());
    }
}
