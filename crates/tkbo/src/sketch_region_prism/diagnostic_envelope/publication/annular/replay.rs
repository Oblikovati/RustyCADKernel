// SPDX-License-Identifier: GPL-2.0-only
//! `Martensite.OwnedAnnularPublicationDiagnostic/v1`: explicit tags, child blocks carry their own
//! versioned codecs; the requested child policies reuse their existing replay encoders.
use super::super::replay::{callback, history_phase_tag, pending_tag, publication_class_tag};
use super::*;
use crate::sketch_region_prism::diagnostic_envelope::replay as shared;
use crate::sketch_region_prism::owned_request::request_replay::encode_annular_publication_options;

impl OwnedAnnularPublicationDiagnostic {
    /// Full typed replay; `diagnostic.replay_bytes()` has no Debug/Display fallback.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"Martensite.OwnedAnnularPublicationDiagnostic/v1\0".to_vec();
        bytes.extend(self.operation.0.to_le_bytes());
        let requested = encode_annular_publication_options(self.requested);
        encode_joined_block(&requested, &mut bytes);
        bytes.push(shared::publication_phase_tag(self.phase));
        bytes.push(publication_class_tag(self.class));
        cause(&self.cause, &mut bytes);
        bytes.extend((self.gaps.len() as u64).to_le_bytes());
        for gap in &self.gaps {
            let PublicationDiagnosticGap::Payload(payload) = gap else {
                unreachable!("annular publication records payload gaps only");
            };
            bytes.push(pending_tag(*payload));
        }
        bytes
    }
}
fn cause(value: &OwnedAnnularPublicationCause, bytes: &mut Vec<u8>) {
    use OwnedAnnularPublicationCause as O;
    match value {
        O::UnsupportedVersion { actual, expected } => {
            bytes.push(0);
            bytes.extend([*actual, *expected].map(u32::to_le_bytes).concat());
        }
        O::Join(join) => block(1, join.encoded(), bytes),
        O::Validation(child) => block(2, child.encoded(), bytes),
        O::History(history) => block(3, history.encoded(), bytes),
        O::HistoryCancelled { operation, phase } => {
            bytes.push(4);
            bytes.extend(operation.0.to_le_bytes());
            bytes.push(history_phase_tag(*phase));
        }
        O::Measures(child) => block(5, child.encoded(), bytes),
        O::Binding { proof } => {
            bytes.push(6);
            encode_joined_text(proof, bytes);
        }
        O::Callback { stop, downstream } => {
            bytes.push(7);
            callback(stop, bytes);
            bytes.push(u8::from(downstream.is_some()));
            if let Some(child) = downstream {
                cause(child, bytes);
            }
        }
        O::Unprojected(payload) => bytes.extend([8, pending_tag(*payload)]),
    }
}
fn block(tag: u8, encoded: &[u8], bytes: &mut Vec<u8>) {
    bytes.push(tag);
    encode_joined_block(encoded, bytes);
}
