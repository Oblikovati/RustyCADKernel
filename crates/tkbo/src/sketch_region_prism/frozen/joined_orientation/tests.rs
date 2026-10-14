// SPDX-License-Identifier: GPL-2.0-only
use super::super::proof_join::operation_tests::JoinFixture;
use super::*;
use crate::sketch_region_prism::{PrismCompletion, PrismImageRole, surface_charts::PrismTrimRole};
#[path = "tests_binding.rs"]
mod binding_tests;
#[path = "tests_refusal.rs"]
mod refusal_tests;
#[path = "tests_replay.rs"]
mod replay_tests;

fn fixture() -> (JoinFixture, JoinedPrismProof) {
    let fixture = JoinFixture::new();
    let join = fixture
        .compose(|| false)
        .ok()
        .expect("admitted actual join");
    (fixture, join)
}
fn certified(join: &JoinedPrismProof) -> JoinedPrismOrientationCertificate {
    inspect_joined_prism_orientation(join, JoinedPrismOrientationOptions::v1(), || false)
        .ok()
        .expect("same-body outward directions")
}
#[test]
fn actual_joined_cubic_has_three_named_faces_and_six_outward_trim_uses() {
    let (fixture, join) = fixture();
    let certificate = certified(&join);
    check_policy(&join, &certificate);
    assert_eq!(certificate.winding().factors(), (1, 1, 1, 1));
    assert_eq!(certificate.jacobian_sign(), 1);
    check_expected_faces(&certificate);
    check_parent_multiplicities(&join, &certificate);
    assert!(matches!(
        fixture.receipt.completion,
        PrismCompletion::Incomplete
    ));
    assert!(!join.projection().is_valid_body());
}
fn check_expected_faces(certificate: &JoinedPrismOrientationCertificate) {
    use Orientation::{Forward as F, Reversed as R};
    assert_eq!(certificate.faces().len(), 3);
    for (role, expected) in [
        (PrismImageRole::First, R),
        (PrismImageRole::Last, F),
        (PrismImageRole::Generated, R),
    ] {
        let row = face_for(certificate, role);
        assert_eq!(row.orientations(), (expected, expected));
    }
    check_side_trims(face_for(certificate, PrismImageRole::Generated));
}
fn check_side_trims(side: &JoinedFaceDirection) {
    use Orientation::{Forward as F, Reversed as R};
    for (role, expected) in [
        (PrismTrimRole::First, F),
        (PrismTrimRole::Last, R),
        (PrismTrimRole::Start, R),
        (PrismTrimRole::End, F),
    ] {
        let trim = trim_for(side, role);
        assert_eq!(trim.orientations(), (expected, expected));
    }
    let start = trim_for(side, PrismTrimRole::Start);
    let end = trim_for(side, PrismTrimRole::End);
    assert_ne!(start.named().occurrence, end.named().occurrence);
}
fn check_parent_multiplicities(
    join: &JoinedPrismProof,
    result: &JoinedPrismOrientationCertificate,
) {
    for face in result.faces() {
        let named = join
            .faces()
            .iter()
            .find(|row| row.occurrence == face.named().occurrence)
            .unwrap();
        assert_eq!(face.named().parents, named.parents);
        assert_dispositions(join, face.parent_dispositions());
        check_named_trims(join, named, face);
    }
}
fn assert_dispositions(join: &JoinedPrismProof, actual: &JoinedParentDispositions) {
    let expected = join
        .parent_dispositions()
        .iter()
        .find(|row| row.occurrence == actual.occurrence)
        .unwrap();
    assert_eq!(actual, expected);
}

fn check_policy(join: &JoinedPrismProof, certificate: &JoinedPrismOrientationCertificate) {
    assert_eq!(certificate.requested(), JoinedPrismOrientationOptions::v1());
    assert_eq!(
        certificate.embedding_requested(),
        join.embedding().requested()
    );
    assert_eq!(
        certificate.evidence_kind(),
        JoinedOrientationEvidenceKind::ExactConditionalDirections
    );
}
fn face_for(
    certificate: &JoinedPrismOrientationCertificate,
    role: PrismImageRole,
) -> &JoinedFaceDirection {
    certificate
        .faces()
        .iter()
        .find(|row| row.named().role == role)
        .expect("one expected authored face")
}
fn trim_for(side: &JoinedFaceDirection, role: PrismTrimRole) -> &JoinedTrimDirection {
    side.trims()
        .iter()
        .find(|row| row.named().role == role)
        .expect("one expected authored trim")
}

fn check_named_trims(join: &JoinedPrismProof, named: &JoinedPrismFace, face: &JoinedFaceDirection) {
    for trim in face.trims() {
        let expected = named
            .trims
            .iter()
            .find(|row| row.occurrence == trim.named().occurrence)
            .unwrap();
        assert_eq!(trim.named(), expected);
        assert_dispositions(join, trim.parent_dispositions());
    }
}

#[path = "tests_domain.rs"]
mod domain_tests;
