// SPDX-License-Identifier: GPL-2.0-only
//! Literal format expectations are independent of the production encoders and actual geometry.
use super::*;
use crate::sketch_region_prism::{
    frozen::{AuthoredPrismId, FrozenPrismKey, FrozenSweepRole, FrozenUseKey},
    owned_request::names::body_map::*,
};
fn key() -> FrozenPrismKey {
    FrozenPrismKey {
        operation: AuthoredPrismId(11),
        source: AuthoredPrismId(13),
        role: FrozenSweepRole::Generated,
        kind: 6,
    }
}
fn occurrence() -> FrozenUseKey {
    let member = key();
    FrozenUseKey(vec![
        (member, AuthoredPrismId(17)),
        (member, AuthoredPrismId(19)),
    ])
}
fn raw_words(values: &[u64]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in values {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}
fn member_bytes() -> Vec<u8> {
    let mut bytes = raw_words(&[11, 13]);
    bytes.extend([3, 6]);
    bytes
}
fn occurrence_bytes() -> Vec<u8> {
    let mut bytes = raw_words(&[2]);
    for token in [17, 19] {
        bytes.extend(member_bytes());
        bytes.extend(raw_words(&[token]));
    }
    bytes
}
fn fixture_slot() -> SemanticNameSlot {
    SemanticNameSlot {
        stage: SemanticNameStage::Output,
        role: SemanticNameRole::Side,
    }
}
fn key_cases() -> Vec<(PrismSemanticMapCause, Vec<u8>)> {
    vec![
        key_cases_0(),
        key_cases_1(),
        key_cases_2(),
        key_cases_3(),
        key_cases_4(),
        key_cases_5(),
        key_cases_6(),
    ]
}
fn key_cases_0() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        MemberMultiplicity {
            key: key(),
            actual: 2,
            domain: 3,
            expected: 1,
        },
        [vec![2], member_bytes(), raw_words(&[2, 3, 1])].concat(),
    )
}
fn key_cases_1() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        UseMultiplicity {
            key: occurrence(),
            actual: 2,
            domain: 3,
            expected: 1,
        },
        [vec![3], occurrence_bytes(), raw_words(&[2, 3, 1])].concat(),
    )
}
fn key_cases_2() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        MemberExpansion { key: key() },
        [vec![4], member_bytes()].concat(),
    )
}
fn key_cases_3() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        UseExpansion { key: occurrence() },
        [vec![5], occurrence_bytes()].concat(),
    )
}
fn key_cases_4() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        MemberCollision {
            first: key(),
            second: key(),
        },
        [vec![6], member_bytes(), member_bytes()].concat(),
    )
}
fn key_cases_5() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        UseCollision {
            first: occurrence(),
            second: occurrence(),
        },
        [vec![7], occurrence_bytes(), occurrence_bytes()].concat(),
    )
}
fn key_cases_6() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        MissingAncestor {
            key: occurrence(),
            parent: occurrence(),
        },
        [vec![8], occurrence_bytes(), occurrence_bytes()].concat(),
    )
}

fn slot_cases() -> Vec<(PrismSemanticMapCause, Vec<u8>)> {
    vec![
        slot_cases_0(),
        slot_cases_1(),
        slot_cases_2(),
        slot_cases_3(),
        slot_cases_4(),
        slot_cases_5(),
        slot_cases_6(),
    ]
}
fn slot_cases_0() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        UnsupportedSourceSlot {
            slot: fixture_slot(),
        },
        vec![9, 9, 20],
    )
}
fn slot_cases_1() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        UnsupportedUseSlot {
            slot: fixture_slot(),
            kind: 6,
        },
        vec![10, 9, 20, 6],
    )
}
fn slot_cases_2() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        OutputSlot {
            key: key(),
            actual: None,
            expected: Some(fixture_slot()),
        },
        [vec![11], member_bytes(), vec![0, 1, 9, 20]].concat(),
    )
}
fn slot_cases_3() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        OutputSlot {
            key: key(),
            actual: Some(fixture_slot()),
            expected: None,
        },
        [vec![11], member_bytes(), vec![1, 9, 20, 0]].concat(),
    )
}
fn slot_cases_4() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        UnsupportedMember { key: key() },
        [vec![12], member_bytes()].concat(),
    )
}
fn slot_cases_5() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (EmptyUse, vec![13])
}
fn slot_cases_6() -> (PrismSemanticMapCause, Vec<u8>) {
    use PrismSemanticMapCause::*;
    (
        EmptyDomain {
            members: 3,
            uses: 7,
        },
        [vec![14], raw_words(&[3, 7])].concat(),
    )
}

#[test]
fn every_map_operand_and_full_ancestry_has_independent_expected_bytes() {
    let mut cases = numeric_cases();
    cases.extend(key_cases());
    cases.extend(slot_cases());
    for (cause, expected) in cases {
        let failure = PrismSemanticMapFailure {
            operation: CallerOperationId(91),
            cause,
        };
        let mut actual = Vec::new();
        names::append(&failure, &mut actual);
        assert_eq!(actual, [raw_words(&[91]), expected].concat());
    }
}
#[test]
fn reservation_codec_retains_slot_counts_and_absence_of_child_class() {
    use crate::sketch_region_prism::owned_request::names::reservations::*;
    let cases = reservation_cases();
    for (cause, expected) in cases {
        let value = PrismReservationFailure {
            operation: CallerOperationId(91),
            cause,
        };
        let mut bytes = Vec::new();
        reservations::append(&value, &mut bytes);
        assert_eq!(bytes, [raw_words(&[91]), expected].concat());
    }
}

fn reservation_cases() -> Vec<(
    crate::sketch_region_prism::owned_request::names::reservations::PrismReservationCause,
    Vec<u8>,
)> {
    vec![
        reservation_cases_0(),
        reservation_cases_1(),
        reservation_cases_2(),
        reservation_cases_3(),
        reservation_cases_4(),
        reservation_cases_5(),
    ]
}
fn reservation_cases_0() -> (
    crate::sketch_region_prism::owned_request::names::reservations::PrismReservationCause,
    Vec<u8>,
) {
    use crate::sketch_region_prism::owned_request::names::reservations::*;
    (PrismReservationCause::ForeignMap, vec![0])
}
fn reservation_cases_1() -> (
    crate::sketch_region_prism::owned_request::names::reservations::PrismReservationCause,
    Vec<u8>,
) {
    use crate::sketch_region_prism::owned_request::names::reservations::*;
    (
        PrismReservationCause::UnsupportedVersion {
            actual: 8,
            expected: 2,
        },
        [
            vec![1],
            8_u32.to_le_bytes().to_vec(),
            2_u32.to_le_bytes().to_vec(),
        ]
        .concat(),
    )
}
fn reservation_cases_2() -> (
    crate::sketch_region_prism::owned_request::names::reservations::PrismReservationCause,
    Vec<u8>,
) {
    use crate::sketch_region_prism::owned_request::names::reservations::*;
    (
        PrismReservationCause::UnknownToken { actual: 77 },
        [vec![2], raw_words(&[77])].concat(),
    )
}
fn reservation_cases_3() -> (
    crate::sketch_region_prism::owned_request::names::reservations::PrismReservationCause,
    Vec<u8>,
) {
    use crate::sketch_region_prism::owned_request::names::reservations::*;
    (PrismReservationCause::EmptyUse, vec![3])
}
fn reservation_cases_4() -> (
    crate::sketch_region_prism::owned_request::names::reservations::PrismReservationCause,
    Vec<u8>,
) {
    use crate::sketch_region_prism::owned_request::names::reservations::*;
    (
        PrismReservationCause::MissingApplication {
            slot: fixture_slot(),
        },
        vec![4, 9, 20],
    )
}
fn reservation_cases_5() -> (
    crate::sketch_region_prism::owned_request::names::reservations::PrismReservationCause,
    Vec<u8>,
) {
    use crate::sketch_region_prism::owned_request::names::reservations::*;
    (
        PrismReservationCause::InapplicableApplied {
            slot: fixture_slot(),
            actual: 4,
        },
        [vec![5, 9, 20], raw_words(&[4])].concat(),
    )
}

fn numeric_cases() -> Vec<(PrismSemanticMapCause, Vec<u8>)> {
    vec![
        (
            PrismSemanticMapCause::UnknownToken { actual: 77 },
            [vec![0], raw_words(&[77])].concat(),
        ),
        (
            PrismSemanticMapCause::Operation {
                actual: 11,
                expected: 13,
            },
            [vec![1], raw_words(&[11, 13])].concat(),
        ),
    ]
}
