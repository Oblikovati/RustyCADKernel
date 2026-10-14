// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::publication::body_view::*;
pub(super) fn append(value: &PrismBodyProjectionFailure, bytes: &mut Vec<u8>) {
    words(&[value.operation.0, value.source.0], bytes);
    bytes.extend(value.version.to_le_bytes());
    class(value.class, bytes);
    entity(value.entity.as_ref(), bytes);
    cause(&value.cause, bytes);
}
pub(super) fn class(value: PrismBodyProjectionClass, bytes: &mut Vec<u8>) {
    match value {
        PrismBodyProjectionClass::Incomplete => bytes.push(0),
    }
}
fn entity(value: Option<&PrismBodyProjectionEntity>, bytes: &mut Vec<u8>) {
    match value {
        None => bytes.push(0),
        Some(PrismBodyProjectionEntity::Member(value)) => {
            bytes.push(1);
            encode_semantic_member_key(bytes, value);
        }
        Some(PrismBodyProjectionEntity::Occurrence(value)) => {
            bytes.push(2);
            encode_semantic_use_key(bytes, value);
        }
    }
}
fn cause(value: &PrismBodyProjectionCause, bytes: &mut Vec<u8>) {
    use PrismBodyProjectionCause::*;
    bytes.push(cause_tag(value));
    match value {
        PublicationBinding => (),
        UnsupportedVersion { actual, expected } => versions(*actual, *expected, bytes),
        SemanticMap(value) => names::append(value, bytes),
        Coverage {
            required,
            actual,
            expected,
        } => coverage(required, *actual, *expected, bytes),
        OperandMismatch { required } => encode_joined_text(required, bytes),
    }
}
fn cause_tag(value: &PrismBodyProjectionCause) -> u8 {
    match value {
        PrismBodyProjectionCause::PublicationBinding => 0,
        PrismBodyProjectionCause::UnsupportedVersion { .. } => 1,
        PrismBodyProjectionCause::SemanticMap(_) => 2,
        PrismBodyProjectionCause::Coverage { .. } => 3,
        PrismBodyProjectionCause::OperandMismatch { .. } => 4,
    }
}
fn coverage(required: &str, actual: usize, expected: usize, bytes: &mut Vec<u8>) {
    encode_joined_text(required, bytes);
    words(&[actual as u64, expected as u64], bytes);
}
