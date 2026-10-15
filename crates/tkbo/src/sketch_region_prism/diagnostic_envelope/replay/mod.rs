// SPDX-License-Identifier: GPL-2.0-only
//! Complete supported V1 envelope bytes include explicit gaps, never unsupported local-owner contents.
mod admission;
mod annular;
mod native;
mod phases;
mod tags;
use super::*;
use crate::sketch_region_prism::frozen::proof_join::{encode_joined_block, encode_joined_text};
pub(in crate::sketch_region_prism) use native::append as append_native_failure;
pub(in crate::sketch_region_prism) use phases::publication as publication_phase_tag;
pub(super) use tags::class as diagnostic_class_tag;
impl OwnedPrismDiagnostic {
    /// Replay supported diagnostic values and explicit coverage; `diagnostic.replay_bytes()` never serializes retained local owners.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"Martensite.OwnedPrismDiagnostic/v1\0".to_vec();
        encode_joined_block(self.context().replay_bytes(), &mut bytes);
        phases::append(self.phase(), &mut bytes);
        bytes.push(tags::class(self.class()));
        bytes.push(u8::from(
            self.coverage() == DiagnosticProjectionCoverage::Incomplete,
        ));
        cause(self.cause(), &mut bytes);
        callback(self.callback(), &mut bytes);
        gaps(self.gaps(), &mut bytes);
        bytes
    }
}
fn cause(value: &OwnedPreparationDiagnosticCause, bytes: &mut Vec<u8>) {
    use OwnedPreparationDiagnosticCause::*;
    match value {
        Admission(value) => {
            bytes.push(0);
            admission::append(value, bytes);
        }
        Native(value) => {
            bytes.push(1);
            native::append(value, bytes);
        }
        Callback => bytes.push(2),
        StageUnavailable { class } => bytes.extend([3, tags::local_class(*class)]),
        MissingLocalState { expected } => {
            bytes.push(4);
            encode_joined_text(expected, bytes);
        }
        Unprojected(value) => bytes.extend([5, tags::payload(*value)]),
    }
}
fn callback(value: Option<&OwnedPreparationCallback>, bytes: &mut Vec<u8>) {
    bytes.push(u8::from(value.is_some()));
    let Some(value) = value else {
        return;
    };
    phases::preparation(value.checkpoint.phase, bytes);
    admission::words(&[value.checkpoint.ordinal as u64], bytes);
    append_callback_cause(&value.cause, bytes);
}
pub(super) fn append_callback_cause(value: &OwnedDiagnosticCallbackCause, bytes: &mut Vec<u8>) {
    match value {
        OwnedDiagnosticCallbackCause::Cancelled => bytes.push(0),
        OwnedDiagnosticCallbackCause::Failed(value) => {
            bytes.push(1);
            native::append(value, bytes);
        }
        OwnedDiagnosticCallbackCause::Exhausted => bytes.push(2),
    }
}
fn gaps(values: &[PrismDiagnosticGap], bytes: &mut Vec<u8>) {
    admission::words(&[values.len() as u64], bytes);
    for value in values {
        gap(value, bytes);
    }
}
fn gap(value: &PrismDiagnosticGap, bytes: &mut Vec<u8>) {
    match value {
        PrismDiagnosticGap::PreparationCause(value) => bytes.extend([0, tags::payload(*value)]),
        PrismDiagnosticGap::OuterOwners(values) => {
            bytes.push(1);
            admission::words(&[values.len() as u64], bytes);
            bytes.extend(values.iter().map(|value| tags::owner(*value)));
        }
        PrismDiagnosticGap::MissingCallbackEvidence => bytes.push(2),
    }
}

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod callback_tests;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod native_tests;
#[cfg(test)]
mod phase_tests;

pub(in crate::sketch_region_prism) use phases::preparation as append_preparation_phase;
