// SPDX-License-Identifier: GPL-2.0-only
//! Full operand binding and achieved exact evidence have deterministic, non-address bytes.
use super::{
    StoredToleranceCap, frames::UsableToleranceFrame, joined_types::*,
    scalar::ExactEndpointCertificate, tolerance_types::*,
};
use crate::sketch_region_prism::{
    frozen::{proof_join::*, *},
    tolerance_inspection::StoredCapDecision,
};
use std::cmp::Ordering;

impl JoinedPrismConformance {
    /// Encode bound operands and every completed proof; example: `proof.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_joined_text("martensite.exact-prism-conformance/v2", &mut bytes);
        encode_joined_block(&self.binding, &mut bytes);
        super::replay_limits::limits(&self.operation_limits, &mut bytes);
        conformance_rows(&self.evidence, &mut bytes);
        bytes
    }
}
impl JoinedPrismTolerance {
    /// Encode the reused exact proof and cap observations; example: `proof.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_joined_text("martensite.exact-prism-tolerance/v2", &mut bytes);
        encode_joined_block(&self.binding, &mut bytes);
        encode_joined_block(&self.conformance, &mut bytes);
        super::replay_limits::limits(&self.operation_limits, &mut bytes);
        cap(self.cap, &mut bytes);
        bytes.extend((self.metadata.len() as u64).to_le_bytes());
        for row in &self.metadata {
            metadata(row, &mut bytes);
        }
        bytes
    }
}
fn conformance_rows(value: &JoinedConformanceEvidence, bytes: &mut Vec<u8>) {
    row_counts(value, bytes);
    for row in &value.placements {
        encode_joined_text(
            &format!("{:?}/{:?}/{:?}", row.entity, row.occurrence, row.role),
            bytes,
        );
        frame(&row.certificate, bytes);
    }
    for row in &value.endpoints {
        endpoint(row, bytes);
    }
    for row in &value.closures {
        closure(row, bytes);
    }
    for row in &value.trims {
        trim(row, bytes);
    }
}
fn row_counts(value: &JoinedConformanceEvidence, bytes: &mut Vec<u8>) {
    for count in [
        value.placements.len(),
        value.endpoints.len(),
        value.closures.len(),
        value.trims.len(),
    ] {
        bytes.extend((count as u64).to_le_bytes());
    }
}
fn trim(row: &JoinedTrimEvidence, bytes: &mut Vec<u8>) {
    encode_joined_text(
        &format!("{:?}/{:?}/{}", row.face, row.occurrence, row.squared_upper),
        bytes,
    );
}
fn frame(value: &UsableToleranceFrame, bytes: &mut Vec<u8>) {
    encode_frozen_placement(value.placement, bytes);
    bytes.push(match value.unscaled_determinant_sign {
        Ordering::Less => 0,
        Ordering::Equal => 1,
        Ordering::Greater => 2,
    });
}
fn scalar(value: &ExactEndpointCertificate, bytes: &mut Vec<u8>) {
    frame(&value.curve_frame, bytes);
    frame(&value.vertex_frame, bytes);
    encode_frozen_numbers([value.requested_spatial, value.effective_spatial], bytes);
    encode_joined_root_options(value.effective_roots, bytes);
    encode_joined_text(&value.proven_squared_upper.to_string(), bytes);
    bytes.extend(value.bisections.to_le_bytes());
}
fn endpoint(value: &JoinedEndpointEvidence, bytes: &mut Vec<u8>) {
    encode_joined_text(
        &format!(
            "{:?}/{:?}/{:?}/{:?}",
            value.edge, value.vertex, value.side, value.contexts
        ),
        bytes,
    );
    encode_frozen_numbers([value.parameter], bytes);
    encode_frozen_numbers(value.local_point, bytes);
    scalar(&value.certificate, bytes);
}
fn closure(value: &JoinedClosureEvidence, bytes: &mut Vec<u8>) {
    encode_joined_text(
        &format!(
            "{:?}/{:?}/{}/{:?}",
            value.contact, value.original, value.stage, value.orientations
        ),
        bytes,
    );
    encode_frozen_numbers(value.local_point, bytes);
    scalar(&value.certificate, bytes);
}
/// Reuse unchanged explicit cap tag/value bytes; `encode_stored_tolerance_cap(cap, bytes)`.
pub(in crate::sketch_region_prism) fn cap(value: StoredToleranceCap, bytes: &mut Vec<u8>) {
    bytes.push(match value {
        StoredToleranceCap::NoLimit => 0,
        StoredToleranceCap::Report(_) => 1,
        StoredToleranceCap::Refuse(_) => 2,
    });
    if let Some(maximum) = value.maximum() {
        encode_frozen_numbers([maximum], bytes);
    }
}
fn metadata(value: &JoinedToleranceMetadata, bytes: &mut Vec<u8>) {
    encode_joined_text(&format!("{:?}/{:?}", value.entity, value.contexts), bytes);
    encode_frozen_numbers([value.observation.raw, value.observation.effective], bytes);
    cap(value.observation.policy, bytes);
    bytes.push(match value.observation.decision {
        StoredCapDecision::Within => 0,
        StoredCapDecision::Reported => 1,
        StoredCapDecision::Refused => 2,
        StoredCapDecision::ExplicitNoLimit => 3,
    });
}
