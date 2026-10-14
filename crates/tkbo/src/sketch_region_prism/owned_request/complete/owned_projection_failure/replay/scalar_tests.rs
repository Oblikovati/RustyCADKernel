use crate::sketch_region_prism::{
    owned_request::OwnedPrismPhase, publication::PrismPublicationPhase,
};
// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::{
    frozen::{AuthoredPrismId, FrozenOriginalKey, joined_volume::*},
    publication::body_view::*,
};
fn words_expected(values: &[u64]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in values {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}
fn volume_header() -> Vec<u8> {
    [
        words_expected(&[17]),
        3_u32.to_le_bytes().to_vec(),
        words_expected(&[19, 23, 29]),
        vec![1],
    ]
    .concat()
}
fn volume_cases() -> Vec<(JoinedPrismVolumeCause, Vec<u8>)> {
    vec![
        volume_cases_0(),
        volume_cases_1(),
        volume_cases_2(),
        volume_cases_3(),
        volume_cases_4(),
        volume_cases_5(),
        volume_cases_6(),
    ]
}
fn volume_cases_0() -> (JoinedPrismVolumeCause, Vec<u8>) {
    (
        JoinedPrismVolumeCause::Version { expected: 1 },
        vec![0, 1, 0, 0, 0],
    )
}
fn volume_cases_1() -> (JoinedPrismVolumeCause, Vec<u8>) {
    (
        JoinedPrismVolumeCause::Binding(VolumeOperand::Orientation),
        vec![1, 0],
    )
}
fn volume_cases_2() -> (JoinedPrismVolumeCause, Vec<u8>) {
    (
        JoinedPrismVolumeCause::Binding(VolumeOperand::Measures),
        vec![1, 1],
    )
}
fn volume_cases_3() -> (JoinedPrismVolumeCause, Vec<u8>) {
    (
        JoinedPrismVolumeCause::Cancelled(VolumeCheckpoint::Begin),
        vec![2, 0],
    )
}
fn volume_cases_4() -> (JoinedPrismVolumeCause, Vec<u8>) {
    (
        JoinedPrismVolumeCause::Cancelled(VolumeCheckpoint::Finish),
        vec![2, 1],
    )
}
fn volume_cases_5() -> (JoinedPrismVolumeCause, Vec<u8>) {
    (
        JoinedPrismVolumeCause::SignFactors(VolumeSignFactors {
            candidate_facing: -1,
            boundary_traversal: 1,
            increasing_parameter: 0,
            jacobian: -1,
            selected_winding: -7,
        }),
        [vec![3, 255, 1, 0, 255], (-7_i64).to_le_bytes().to_vec()].concat(),
    )
}
fn volume_cases_6() -> (JoinedPrismVolumeCause, Vec<u8>) {
    (
        JoinedPrismVolumeCause::ScalarContradiction {
            signed_enclosure: [-0., f64::from_bits(0x7ff8_0000_0000_0123)],
            exact_sign: -1,
        },
        [
            vec![4],
            words_expected(&[1 << 63, 0x7ff8_0000_0000_0123]),
            vec![255],
        ]
        .concat(),
    )
}

#[test]
fn volume_failure_preserves_complete_header_signs_signed_zero_and_nan_bits() {
    for (cause, expected) in volume_cases() {
        let value = JoinedPrismVolumeFailure {
            operation: AuthoredPrismId(17),
            requested_version: 3,
            entity: FrozenOriginalKey {
                source: 19,
                member: AuthoredPrismId(23),
                directed_use: AuthoredPrismId(29),
            },
            class: JoinedVolumeFailureClass::Cancelled,
            cause,
        };
        let mut bytes = Vec::new();
        volume::append(&value, &mut bytes);
        assert_eq!(bytes, [volume_header(), expected].concat());
    }
}
fn body_cases() -> Vec<(PrismBodyProjectionCause, Vec<u8>)> {
    vec![
        (PrismBodyProjectionCause::PublicationBinding,vec![0]),
        (PrismBodyProjectionCause::UnsupportedVersion{actual:3,expected:1},vec![1,3,0,0,0,1,0,0,0]),
        (PrismBodyProjectionCause::Coverage{required:"rim",actual:2,expected:6},[vec![3],words_expected(&[3]),b"rim".to_vec(),words_expected(&[2,6])].concat()),
        (PrismBodyProjectionCause::OperandMismatch{required:"chart"},[vec![4],words_expected(&[5]),b"chart".to_vec()].concat()),
        (PrismBodyProjectionCause::SemanticMap(Box::new(crate::sketch_region_prism::owned_request::names::body_map::PrismSemanticMapFailure{operation:CallerOperationId(31),cause:crate::sketch_region_prism::owned_request::names::body_map::PrismSemanticMapCause::EmptyUse})),[vec![2],words_expected(&[31]),vec![13]].concat()),
    ]
}
#[test]
fn body_failure_retains_caller_header_typed_causes_and_nested_map() {
    for (cause, expected) in body_cases() {
        let value = PrismBodyProjectionFailure {
            operation: CallerOperationId(17),
            source: CallerBoundaryId(19),
            version: 3,
            entity: None,
            class: PrismBodyProjectionClass::Incomplete,
            cause,
        };
        let mut bytes = Vec::new();
        body::append(&value, &mut bytes);
        assert_eq!(
            bytes,
            [words_expected(&[17, 19]), vec![3, 0, 0, 0, 0, 0], expected].concat()
        );
    }
}
#[test]
fn callback_optional_fields_and_all_phase_tags_preserve_exact_checkpoint() {
    let cases = callback_phases();
    for (phase, expected) in cases {
        assert_callback(phase, expected);
    }
    let mut absent = Vec::new();
    callback::append(None, &mut absent);
    assert_eq!(absent, [0]);
}

fn assert_callback(phase: CompletePrismPhase, expected: Vec<u8>) {
    let value = CompletePrismCallbackStop {
        checkpoint: CompletePrismCheckpoint {
            phase,
            ordinal: 13,
            child_ordinal: Some(7),
        },
        cause: CompletePrismCallbackCause::Exhausted,
    };
    let mut bytes = Vec::new();
    callback::append(Some(&value), &mut bytes);
    assert_eq!(bytes, expected_callback(expected));
}

fn expected_callback(phase: Vec<u8>) -> Vec<u8> {
    let checkpoint = [words_expected(&[13]), vec![1], words_expected(&[7])].concat();
    [vec![1], phase, checkpoint, vec![2]].concat()
}

#[test]
fn body_entity_codec_keeps_full_semantic_member_and_directed_ancestry() {
    let member = semantic_member();
    let step = semantic_step(member.clone());
    let member_bytes = [words_expected(&[11, 13]), vec![2, 3, 2, 6]].concat();
    let use_bytes = expected_semantic_use(&member_bytes);
    assert_body_entity(
        PrismBodyProjectionEntity::Member(member),
        [vec![1], member_bytes].concat(),
    );
    let occurrence = SemanticUseKey(vec![step.clone(), step]);
    assert_body_entity(
        PrismBodyProjectionEntity::Occurrence(occurrence),
        [vec![2], use_bytes].concat(),
    );
}
fn semantic_member() -> SemanticMemberKey {
    SemanticMemberKey {
        operation: CallerOperationId(11),
        source: CallerBoundaryId(13),
        source_member: SemanticNameSlot {
            stage: SemanticNameStage::Original,
            role: SemanticNameRole::Boundary,
        },
        sweep_role: crate::sketch_region_prism::frozen::FrozenSweepRole::Last,
        kind: 6,
    }
}
fn assert_body_entity(entity: PrismBodyProjectionEntity, expected: Vec<u8>) {
    let value = body_with_entity(entity);
    let mut bytes = Vec::new();
    body::append(&value, &mut bytes);
    assert_eq!(
        bytes,
        [
            words_expected(&[17, 19]),
            vec![3, 0, 0, 0, 0],
            expected,
            vec![0]
        ]
        .concat()
    );
}
fn body_with_entity(entity: PrismBodyProjectionEntity) -> PrismBodyProjectionFailure {
    PrismBodyProjectionFailure {
        operation: CallerOperationId(17),
        source: CallerBoundaryId(19),
        version: 3,
        entity: Some(entity),
        class: PrismBodyProjectionClass::Incomplete,
        cause: PrismBodyProjectionCause::PublicationBinding,
    }
}

fn semantic_step(member: SemanticMemberKey) -> SemanticUseStep {
    SemanticUseStep {
        member,
        directed_use: SemanticNameSlot {
            stage: SemanticNameStage::Output,
            role: SemanticNameRole::FaceUse,
        },
    }
}

fn expected_semantic_use(member: &[u8]) -> Vec<u8> {
    let step = [member, &[9, 28]].concat();
    [words_expected(&[2]), step.clone(), step].concat()
}

fn callback_phases() -> Vec<(CompletePrismPhase, Vec<u8>)> {
    vec![
        (
            CompletePrismPhase::Preparation(OwnedPrismPhase::Construction),
            vec![0, 1],
        ),
        (CompletePrismPhase::Resolution, vec![1]),
        (
            CompletePrismPhase::Publication(PrismPublicationPhase::Measures),
            vec![2, 4],
        ),
        (CompletePrismPhase::Names, vec![3]),
        (CompletePrismPhase::Reservations, vec![4]),
        (CompletePrismPhase::Body, vec![5]),
        (CompletePrismPhase::Volume, vec![6]),
        (CompletePrismPhase::Commit, vec![7]),
    ]
}
