// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn unsupported_behavior_refuses_before_callbacks_and_preserves_request() {
    let (_, join) = fixture();
    let requested = JoinedPrismOrientationOptions { version: 0 };
    let mut polls = 0;
    let failure = inspect_joined_prism_orientation(&join, requested, || {
        polls += 1;
        false
    })
    .err()
    .unwrap();
    assert_eq!(polls, 0);
    assert_eq!(failure.operation, AuthoredPrismId(700));
    assert_eq!(failure.requested, requested);
    assert_eq!(failure.phase, JoinedPrismOrientationPhase::Admission);
    assert_version(failure.cause);
}
fn assert_version(cause: JoinedPrismOrientationCause) {
    assert!(matches!(
        cause,
        JoinedPrismOrientationCause::Version {
            actual: 0,
            expected: 1
        }
    ));
}
#[test]
fn both_fixed_checkpoints_cancel_without_partial_success_or_poisoned_reuse() {
    let (_, join) = fixture();
    let before = certified(&join).replay_bytes();
    for (cutoff, phase) in [
        (1, JoinedPrismOrientationPhase::Admission),
        (2, JoinedPrismOrientationPhase::Commit),
    ] {
        let failure = cancelled_once(&join, cutoff);
        assert_eq!(failure.phase, phase);
        assert!(matches!(
            failure.cause,
            JoinedPrismOrientationCause::Cancelled
        ));
        assert_eq!(certified(&join).replay_bytes(), before);
    }
}
fn cancelled_once(join: &JoinedPrismProof, cutoff: usize) -> JoinedPrismOrientationFailure {
    let mut polls = 0;
    let failure =
        inspect_joined_prism_orientation(join, JoinedPrismOrientationOptions::v1(), || {
            polls += 1;
            polls == cutoff
        })
        .err()
        .unwrap();
    assert_eq!(polls, cutoff);
    failure
}
#[test]
fn isolated_missing_and_duplicate_face_roles_refuse_authored_table_lookup() {
    let (_, join) = fixture();
    let mut rows = join.faces().to_vec();
    let removed = rows.pop().unwrap();
    assert_count(faces::named_face(&rows, removed.role), 0);
    rows.push(removed.clone());
    rows.push(removed.clone());
    assert_count(faces::named_face(&rows, removed.role), 2);
}
#[test]
fn isolated_missing_and_duplicate_seam_roles_refuse_authored_table_lookup() {
    let (_, join) = fixture();
    let mut face = faces::named_face(join.faces(), PrismImageRole::Generated)
        .ok()
        .unwrap()
        .clone();
    let index = face
        .trims
        .iter()
        .position(|row| row.role == PrismTrimRole::Start)
        .unwrap();
    let removed = face.trims.remove(index);
    assert_count(trims::named_trim(&face, PrismTrimRole::Start), 0);
    face.trims.push(removed.clone());
    face.trims.push(removed);
    assert_count(trims::named_trim(&face, PrismTrimRole::Start), 2);
}
fn assert_count<T>(result: JoinedDirectionCheck<T>, expected_count: usize) {
    assert!(
        matches!(result, Err(JoinedPrismOrientationCause::Count { actual, expected: 1, .. })
        if actual == expected_count)
    );
}
