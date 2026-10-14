// SPDX-License-Identifier: GPL-2.0-only
//! Complete join bytes and all achieved evidence use exact numeric bits and semantic source keys.
use super::*;
use crate::sketch_region_prism::{
    exact_tolerance::{replay as shared, replay_limits},
    frozen::proof_join::*,
};
impl AnnularPrismConformance {
    /// Encode immutable source-bound evidence without addresses; `proof.replay_bytes()`.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_joined_text("martensite.annular-exact-conformance/v2", &mut bytes);
        encode_joined_block(&self.binding, &mut bytes);
        replay_limits::limits(&self.limits, &mut bytes);
        shared::conformance_rows(&self.evidence.body, &mut bytes);
        upstream(&self.evidence, &mut bytes);
        bytes
    }
}
impl AnnularPrismTolerance {
    /// Retain unchanged cap, conformance and all metadata; `proof.replay_bytes()`.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_joined_text("martensite.annular-exact-tolerance/v2", &mut bytes);
        encode_joined_block(&self.binding, &mut bytes);
        encode_joined_block(&self.conformance, &mut bytes);
        replay_limits::limits(&self.limits, &mut bytes);
        shared::cap(self.cap, &mut bytes);
        bytes.extend((self.metadata.len() as u64).to_le_bytes());
        for row in &self.metadata {
            shared::metadata(row, &mut bytes);
        }
        bytes
    }
}
fn upstream(evidence: &AnnularConformanceEvidence, bytes: &mut Vec<u8>) {
    bytes.extend((evidence.supports.len() as u64).to_le_bytes());
    for row in &evidence.supports {
        encode_joined_text(&format!("{:?}", row.context), bytes);
        shared::frame(&row.frame, bytes);
    }
    bytes.extend((evidence.closures.len() as u64).to_le_bytes());
    for row in &evidence.closures {
        closure(row, bytes);
    }
    bytes.extend((evidence.source_reports.len() as u64).to_le_bytes());
    for row in &evidence.source_reports {
        report(row, bytes);
    }
}
fn closure(row: &AnnularClosureEvidence, bytes: &mut Vec<u8>) {
    encode_joined_text(
        &format!(
            "{:?}/{:?}/{:?}",
            row.context, row.reused_from, row.orientation
        ),
        bytes,
    );
    encode_frozen_numbers([row.parameter, row.source_raw_tolerance], bytes);
    encode_frozen_numbers(row.local_point, bytes);
    shared::scalar(&row.certificate, bytes);
}
fn report(row: &AnnularSourceReport, bytes: &mut Vec<u8>) {
    encode_joined_text(
        &format!("{:?}/{:?}/{:?}", row.source, row.stage, row.residual),
        bytes,
    );
    encode_frozen_numbers([row.requested_limit, row.effective_limit], bytes);
    encode_joined_root_options(row.effective_roots, bytes);
    bytes.extend(row.bisections.to_le_bytes());
}
