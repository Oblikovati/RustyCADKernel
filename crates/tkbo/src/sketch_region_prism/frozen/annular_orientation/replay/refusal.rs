// SPDX-License-Identifier: GPL-2.0-only
use super::*;
impl AnnularDirectionFailure {
    /// Encode original policy, phase and exact cause; `failure.replay_bytes()` contains no native address.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"Martensite.AnnularOrientationFailure\0".to_vec();
        word(&mut bytes, self.operation.0);
        bytes.extend(self.requested.version.to_le_bytes());
        bytes.push(match self.phase {
            AnnularDirectionPhase::Admission => 1,
            AnnularDirectionPhase::Winding => 2,
            AnnularDirectionPhase::Directions => 3,
            AnnularDirectionPhase::Commit => 4,
        });
        block(&mut bytes, &self.source_binding);
        cause(&mut bytes, &self.cause);
        bytes
    }
}
fn cause(bytes: &mut Vec<u8>, value: &AnnularDirectionCause) {
    match value {
        AnnularDirectionCause::Version { actual, expected } => version(bytes, *actual, *expected),
        AnnularDirectionCause::Count {
            subject,
            actual,
            expected,
        } => counted(bytes, subject, *actual, *expected),
        AnnularDirectionCause::Winding { source, actual } => winding(bytes, *source, *actual),
        other => directional_cause(bytes, other),
    }
}
fn version(bytes: &mut Vec<u8>, actual: u32, expected: u32) {
    bytes.push(1);
    bytes.extend(actual.to_le_bytes());
    bytes.extend(expected.to_le_bytes());
}
fn counted(bytes: &mut Vec<u8>, subject: &str, actual: usize, expected: usize) {
    bytes.push(2);
    block(bytes, subject.as_bytes());
    word(bytes, actual as u64);
    word(bytes, expected as u64);
}
fn winding(bytes: &mut Vec<u8>, source: RegionSourceId, actual: Option<i64>) {
    bytes.push(3);
    word(bytes, source.0);
    bytes.push(u8::from(actual.is_some()));
    if let Some(value) = actual {
        bytes.extend(value.to_le_bytes());
    }
}
fn directions(bytes: &mut Vec<u8>, tag: u8, key: &FrozenUseKey, values: &[Orientation]) {
    bytes.push(tag);
    occurrence(bytes, key);
    for value in values {
        direction(bytes, *value);
    }
}
fn context(bytes: &mut Vec<u8>, key: &FrozenUseKey, required: &str) {
    bytes.push(6);
    occurrence(bytes, key);
    block(bytes, required.as_bytes());
}

fn directional_cause(bytes: &mut Vec<u8>, value: &AnnularDirectionCause) {
    match value {
        AnnularDirectionCause::Unsupported {
            occurrence: key,
            actual,
        } => directions(bytes, 4, key, &[*actual]),
        AnnularDirectionCause::Direction {
            occurrence: key,
            actual,
            expected,
        } => directions(bytes, 5, key, &[*actual, *expected]),
        AnnularDirectionCause::Context {
            occurrence: key,
            required,
        } => context(bytes, key, required),
        AnnularDirectionCause::ZeroJacobian => bytes.push(7),
        AnnularDirectionCause::Cancelled => bytes.push(8),
        _ => unreachable!("scalar cause encoded before directional cause"),
    }
}
