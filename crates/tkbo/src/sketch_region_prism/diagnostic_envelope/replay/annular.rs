// SPDX-License-Identifier: GPL-2.0-only
//! `Martensite.OwnedAnnularPreparationDiagnostic/v1`: explicit tags only, reusing the V1
//! preparation cause, callback and gap encoders unchanged; retained owners are never serialized.
use super::super::annular_preparation::*;
use super::*;
use crate::sketch_region_prism::owned_request::annular::AnnularAdmissionCause;
use crate::sketch_region_prism::owned_request::request_replay::encode_owned_annular_request;

impl OwnedAnnularPreparationDiagnostic {
    /// Full typed replay; `diagnostic.replay_bytes()` has no Debug/Display fallback.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"Martensite.OwnedAnnularPreparationDiagnostic/v1\0".to_vec();
        encode_joined_block(&encode_owned_annular_request(self.request()), &mut bytes);
        phases::preparation(self.phase(), &mut bytes);
        bytes.push(tags::class(self.class()));
        let incomplete = self.coverage() == DiagnosticProjectionCoverage::Incomplete;
        bytes.push(u8::from(incomplete));
        annular_cause(self.cause(), &mut bytes);
        callback(self.callback(), &mut bytes);
        gaps(self.gaps(), &mut bytes);
        bytes
    }
}
fn annular_cause(value: &OwnedAnnularPreparationCause, bytes: &mut Vec<u8>) {
    match value {
        OwnedAnnularPreparationCause::Admission(AnnularAdmissionCause::Owned(value)) => {
            bytes.push(0);
            admission::append(value, bytes);
        }
        OwnedAnnularPreparationCause::Admission(AnnularAdmissionCause::Names { quantity }) => {
            bytes.push(1);
            encode_joined_text(quantity, bytes);
        }
        OwnedAnnularPreparationCause::Role {
            required,
            actual,
            expected,
        } => {
            bytes.push(2);
            encode_joined_text(required, bytes);
            admission::words(&[*actual as u64, *expected as u64], bytes);
        }
        OwnedAnnularPreparationCause::Stage(value) => {
            bytes.push(3);
            cause(value, bytes);
        }
    }
}
