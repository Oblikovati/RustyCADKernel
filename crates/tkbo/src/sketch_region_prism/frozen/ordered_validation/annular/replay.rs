// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::{
    frozen::proof_join::{encode_joined_block, encode_joined_text},
    owned_request::names::encode_frozen_member_key,
};
impl AnnularValidationCertificate {
    /// Replay each complete child proof and explicit stage order; `proof.replay_bytes()` includes original operands.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_joined_text("Martensite.OrderedAnnularValidation/v2", &mut bytes);
        bytes.extend(self.requested.version.to_le_bytes());
        bytes.push(0); // ExactTwoBoundaryPolynomialPrism.
        trace(&self.trace, &mut bytes);
        encode_joined_block(&self.source, &mut bytes);
        for child in self.child_bytes() {
            encode_joined_block(&child, &mut bytes);
        }
        bytes
    }
    fn child_bytes(&self) -> [Vec<u8>; 5] {
        [
            topology(self.topology()),
            self.orientation().replay_bytes(),
            self.conformance().replay_bytes(),
            self.intersection().replay_bytes(),
            self.tolerance().replay_bytes(),
        ]
    }
}
impl AnnularIntersectionCertificate {
    /// Preserve the actual sealed theorem and its full join; `proof.replay_bytes()` adds no geometric recomputation.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_joined_text("Martensite.AnnularTableCorrespondence/v2", &mut bytes);
        encode_joined_block(&self.source, &mut bytes);
        encode_joined_block(&self.embedding, &mut bytes);
        bytes
    }
}
fn topology(proof: &AnnularTopologyCertificate) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_joined_text("Martensite.AnnularTopology/v2", &mut bytes);
    encode_joined_block(proof.source_bytes(), &mut bytes);
    bytes.extend(proof.operation().0.to_le_bytes());
    encode_frozen_member_key(&mut bytes, proof.root());
    for count in proof.counts() {
        bytes.extend(count.to_le_bytes());
    }
    bytes.extend([0, 0]); // ExactCombinatorial; numerical error NotApplicable.
    bytes
}
fn trace(trace: &OrderedPrismStageTrace, bytes: &mut Vec<u8>) {
    for stage in OrderedPrismStage::ALL {
        bytes.push(match trace.status(stage) {
            OrderedPrismStageStatus::Blocked => 0,
            OrderedPrismStageStatus::Passed => 1,
            OrderedPrismStageStatus::Failed => 2,
            OrderedPrismStageStatus::Incomplete => 3,
            OrderedPrismStageStatus::Cancelled => 4,
        });
    }
}
