// SPDX-License-Identifier: GPL-2.0-only
//! Tagged conclusions retain full named contexts beside the exact immutable input bytes.
use super::*;
use crate::sketch_region_prism::owned_request::names::{
    encode_frozen_member_key as member, encode_frozen_use_key as occurrence,
};
mod refusal;
mod rows;
impl AnnularOrientationCertificate {
    /// Recompute complete portable evidence; `proof.replay_bytes()` includes all retained conclusions.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"Martensite.AnnularOrientation\0".to_vec();
        bytes.extend(self.requested.version.to_le_bytes());
        block(&mut bytes, &self.binding);
        bytes.push(self.jacobian_sign as u8);
        for boundary in &self.boundaries {
            rows::boundary(&mut bytes, boundary);
        }
        word(&mut bytes, self.faces.len() as u64);
        for face in &self.faces {
            rows::face(&mut bytes, face);
        }
        bytes
    }
}
fn word(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend(value.to_le_bytes());
}
fn block(bytes: &mut Vec<u8>, value: &[u8]) {
    word(bytes, value.len() as u64);
    bytes.extend(value);
}
fn direction(bytes: &mut Vec<u8>, value: Orientation) {
    bytes.push(match value {
        Orientation::Forward => 1,
        Orientation::Reversed => 2,
        Orientation::Internal => 3,
        Orientation::External => 4,
    });
}
fn parents(bytes: &mut Vec<u8>, values: &[FrozenOriginalKey]) {
    word(bytes, values.len() as u64);
    for value in values {
        word(bytes, value.source);
        word(bytes, value.member.0);
        word(bytes, value.directed_use.0);
    }
}
fn dispositions(bytes: &mut Vec<u8>, value: &JoinedParentDispositions) {
    occurrence(bytes, &value.occurrence);
    parents(bytes, &value.preserved);
    parents(bytes, &value.modified);
    parents(bytes, &value.generated);
}
fn image_role(bytes: &mut Vec<u8>, role: PrismImageRole) {
    bytes.push(match role {
        PrismImageRole::First => 1,
        PrismImageRole::Last => 2,
        PrismImageRole::Generated => 3,
    });
}
fn trim_role(bytes: &mut Vec<u8>, role: PrismTrimRole) {
    bytes.push(match role {
        PrismTrimRole::First => 1,
        PrismTrimRole::Last => 2,
        PrismTrimRole::Start => 3,
        PrismTrimRole::End => 4,
    });
}
