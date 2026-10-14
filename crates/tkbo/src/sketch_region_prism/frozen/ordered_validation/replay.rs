// SPDX-License-Identifier: GPL-2.0-only
//! Child encoders retain exact operands; native allocation identities are never formatted here.
use super::super::proof_join::{encode_joined_block, encode_joined_text};
use super::*;
impl OrderedPrismValidationCertificate {
    /// Replay complete validation evidence, including original mapped findings; example: `proof.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_joined_text("martensite.ordered-joined-prism-validation/v1", &mut bytes);
        bytes.extend(self.requested.version.to_le_bytes());
        encode_joined_text(&format!("{:?}/{:?}", self.mode(), self.trace), &mut bytes);
        encode_joined_block(&self.source, &mut bytes);
        for proof in [
            self.topology.replay_bytes(),
            self.orientation.replay_bytes(),
            self.conformance.replay_bytes(),
            self.intersection.replay_bytes(),
            self.tolerance.replay_bytes(),
        ] {
            encode_joined_block(&proof, &mut bytes);
        }
        bytes
    }
}
impl JoinedPrismIntersectionCertificate {
    /// The joined encoder already names every domain, trim and conclusion; example: `proof.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_joined_text(
            "martensite.exact-joined-prism-incidence-table/v1",
            &mut bytes,
        );
        encode_joined_block(&self.source, &mut bytes);
        bytes
    }
}
