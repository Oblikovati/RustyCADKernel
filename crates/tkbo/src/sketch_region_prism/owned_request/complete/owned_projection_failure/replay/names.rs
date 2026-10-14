// SPDX-License-Identifier: GPL-2.0-only
//! Failed mapping grants no caller qualification to these complete authored observations.
use super::*;
use crate::sketch_region_prism::owned_request::names::body_map::*;
pub(super) fn append(value: &PrismSemanticMapFailure, bytes: &mut Vec<u8>) {
    words(&[value.operation.0], bytes);
    encode_cause(&value.cause, bytes);
}
fn encode_cause(value: &PrismSemanticMapCause, bytes: &mut Vec<u8>) {
    use PrismSemanticMapCause::*;
    match value {
        UnknownToken { .. } | Operation { .. } | EmptyDomain { .. } => numeric(value, bytes),
        MemberMultiplicity { .. } | UseMultiplicity { .. } => multiplicity(value, bytes),
        MemberExpansion { .. } | UseExpansion { .. } | UnsupportedMember { .. } => {
            single(value, bytes)
        }
        MemberCollision { .. } | UseCollision { .. } | MissingAncestor { .. } => {
            paired(value, bytes)
        }
        UnsupportedSourceSlot { .. } | UnsupportedUseSlot { .. } => slots(value, bytes),
        OutputSlot {
            key,
            actual,
            expected,
        } => output(*key, *actual, *expected, bytes),
        EmptyUse => bytes.push(13),
    }
}
fn numeric(value: &PrismSemanticMapCause, bytes: &mut Vec<u8>) {
    use PrismSemanticMapCause::*;
    let (tag, values) = match value {
        UnknownToken { actual } => (0, vec![*actual]),
        Operation { actual, expected } => (1, vec![*actual, *expected]),
        EmptyDomain { members, uses } => (14, vec![*members as u64, *uses as u64]),
        _ => unreachable!("numeric map cause selected by exhaustive dispatch"),
    };
    bytes.push(tag);
    words(&values, bytes);
}
fn multiplicity(value: &PrismSemanticMapCause, bytes: &mut Vec<u8>) {
    use PrismSemanticMapCause::*;
    match value {
        MemberMultiplicity { key, .. } => member(2, *key, bytes),
        UseMultiplicity { key, .. } => directed(3, key, bytes),
        _ => unreachable!("multiplicity selected by exhaustive dispatch"),
    }
    words(&multiplicity_counts(value), bytes);
}
fn multiplicity_counts(value: &PrismSemanticMapCause) -> [u64; 3] {
    use PrismSemanticMapCause::*;
    let (MemberMultiplicity {
        actual,
        domain,
        expected,
        ..
    }
    | UseMultiplicity {
        actual,
        domain,
        expected,
        ..
    }) = value
    else {
        unreachable!("multiplicity operands required");
    };
    [*actual as u64, *domain as u64, *expected as u64]
}
fn single(value: &PrismSemanticMapCause, bytes: &mut Vec<u8>) {
    use PrismSemanticMapCause::*;
    match value {
        MemberExpansion { key } => member(4, *key, bytes),
        UseExpansion { key } => directed(5, key, bytes),
        UnsupportedMember { key } => member(12, *key, bytes),
        _ => unreachable!("single key selected by exhaustive dispatch"),
    }
}
fn paired(value: &PrismSemanticMapCause, bytes: &mut Vec<u8>) {
    use PrismSemanticMapCause::*;
    match value {
        MemberCollision { first, second } => {
            member(6, *first, bytes);
            encode_frozen_member_key(bytes, *second);
        }
        UseCollision { first, second } => pair_uses(7, first, second, bytes),
        MissingAncestor { key, parent } => pair_uses(8, key, parent, bytes),
        _ => unreachable!("paired keys selected by exhaustive dispatch"),
    }
}
fn slots(value: &PrismSemanticMapCause, bytes: &mut Vec<u8>) {
    use PrismSemanticMapCause::*;
    match value {
        UnsupportedSourceSlot { slot: value } => {
            bytes.push(9);
            slot(*value, bytes);
        }
        UnsupportedUseSlot { slot: value, kind } => {
            bytes.push(10);
            slot(*value, bytes);
            bytes.push(*kind);
        }
        _ => unreachable!("slot fields selected by exhaustive dispatch"),
    }
}
fn output(
    key: crate::sketch_region_prism::frozen::FrozenPrismKey,
    actual: Option<SemanticNameSlot>,
    expected: Option<SemanticNameSlot>,
    bytes: &mut Vec<u8>,
) {
    member(11, key, bytes);
    optional_slot(actual, bytes);
    optional_slot(expected, bytes);
}
fn member(tag: u8, key: crate::sketch_region_prism::frozen::FrozenPrismKey, bytes: &mut Vec<u8>) {
    bytes.push(tag);
    encode_frozen_member_key(bytes, key);
}
fn directed(tag: u8, key: &crate::sketch_region_prism::frozen::FrozenUseKey, bytes: &mut Vec<u8>) {
    bytes.push(tag);
    encode_frozen_use_key(bytes, key);
}
fn pair_uses(
    tag: u8,
    first: &crate::sketch_region_prism::frozen::FrozenUseKey,
    second: &crate::sketch_region_prism::frozen::FrozenUseKey,
    bytes: &mut Vec<u8>,
) {
    directed(tag, first, bytes);
    encode_frozen_use_key(bytes, second);
}
