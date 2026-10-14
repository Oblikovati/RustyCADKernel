// SPDX-License-Identifier: GPL-2.0-only
//! An owned name envelope preserves opaque child bytes; it is not a geometry certificate.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SemanticReplayEnvelope {
    plan: SingleBoundaryNamePlan,
    child: Vec<u8>,
    replay: Vec<u8>,
}
impl SemanticReplayEnvelope {
    /// Own names and unmodified opaque child bytes; example: `SemanticReplayEnvelope::new(plan, bytes)`.
    pub(crate) fn new(plan: SingleBoundaryNamePlan, child: &[u8]) -> Self {
        let replay = encode_envelope(&plan, child);
        Self {
            plan,
            child: child.to_vec(),
            replay,
        }
    }
    /// Full names are part of comparison; example: `envelope.matches_plan(&plan)`.
    pub(crate) fn matches_plan(&self, plan: &SingleBoundaryNamePlan) -> bool {
        &self.plan == plan
    }
    /// Borrow immutable name declarations; example: `envelope.plan().entries()`.
    pub(crate) fn plan(&self) -> &SingleBoundaryNamePlan {
        &self.plan
    }
    /// Legacy payload is unchanged; example: `envelope.child_bytes()`.
    pub(crate) fn child_bytes(&self) -> &[u8] {
        &self.child
    }
    /// Full bytes include caller identities, intent, dictionary and child; example: `envelope.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
}
fn encode_envelope(plan: &SingleBoundaryNamePlan, child: &[u8]) -> Vec<u8> {
    let mut bytes = b"Martensite.SemanticNames\0".to_vec();
    bytes.extend_from_slice(&plan.version().to_le_bytes());
    encode_request(&mut bytes, plan);
    word(&mut bytes, plan.entries().len() as u64);
    for entry in plan.entries() {
        encode_entry(&mut bytes, entry);
    }
    word(&mut bytes, child.len() as u64);
    bytes.extend_from_slice(child);
    bytes
}
fn encode_request(bytes: &mut Vec<u8>, plan: &SingleBoundaryNamePlan) {
    let ids = plan.identities();
    for value in [
        ids.operation.0,
        ids.boundary.0,
        ids.closure.0,
        ids.start.0,
        ids.end.0,
    ] {
        word(bytes, value);
    }
    bytes.push(1); // V1 BoundedRegionContaining discriminant, independent of Rust enum layout.
    let SemanticSelectionIntent::BoundedRegionContaining { seed } = plan.selection();
    for value in seed {
        word(bytes, value.to_bits());
    }
}
fn encode_entry(bytes: &mut Vec<u8>, entry: &SemanticNameEntry) {
    word(bytes, entry.token.value());
    bytes.extend_from_slice(&[entry.slot.stage as u8, entry.slot.role as u8]);
    word(bytes, entry.operation.0);
    word(bytes, entry.source.0);
    let (tag, value) = match entry.anchor {
        SemanticCallerAnchor::Operation(value) => (1, value.0),
        SemanticCallerAnchor::Boundary(value) => (2, value.0),
        SemanticCallerAnchor::Closure(value) => (3, value.0),
        SemanticCallerAnchor::Start(value) => (4, value.0),
        SemanticCallerAnchor::End(value) => (5, value.0),
    };
    bytes.push(tag);
    word(bytes, value);
}
pub(super) fn word(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
