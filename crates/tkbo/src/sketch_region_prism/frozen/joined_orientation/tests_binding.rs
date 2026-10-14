// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn equal_geometry_with_different_authored_operation_cannot_borrow_certificate() {
    let (_, original) = fixture();
    let certificate = certified(&original);
    let mut renamed = JoinFixture::new();
    renamed.authored.operation = AuthoredPrismId(1700);
    let changed = renamed.compose(|| false).ok().unwrap();
    assert!(certificate.matches_join(&original));
    assert_eq!(certificate.binding_bytes(), original.replay_bytes());
    assert!(!certificate.matches_join(&changed));
    assert_ne!(
        certificate.replay_bytes(),
        certified(&changed).replay_bytes()
    );
}
#[test]
fn authored_binding_permutations_preserve_complete_named_replay() {
    let (mut fixture, original) = fixture();
    fixture.authored.originals.reverse();
    fixture.authored.sources.reverse();
    fixture.authored.directed.reverse();
    let reordered = fixture.compose(|| false).ok().unwrap();
    assert_eq!(original.replay_bytes(), reordered.replay_bytes());
    assert_eq!(
        certified(&original).replay_bytes(),
        certified(&reordered).replay_bytes()
    );
}
#[test]
fn mutable_source_changes_do_not_change_owned_join_or_directions() {
    use crate::sketch_region_prism::region_certificate::orientation::tests::mutate_orientation_fixture as edit_target;
    for target in 0..3 {
        let (fixture, join) = fixture();
        let certificate = certified(&join);
        let before = certificate.replay_bytes();
        edit_target(target, &fixture.shapes, &fixture.receipt);
        assert_eq!(certified(&join).replay_bytes(), before);
        assert_eq!(certificate.replay_bytes(), before);
        assert!(fixture.rebind(&join).is_err());
    }
}
#[test]
fn callback_source_change_keeps_owned_math_and_requires_separate_live_rebind() {
    use crate::sketch_region_prism::region_certificate::orientation::tests::mutate_orientation_fixture as edit_target;
    let (fixture, join) = fixture();
    let before = certified(&join).replay_bytes();
    let mut polls = 0;
    let result =
        inspect_joined_prism_orientation(&join, JoinedPrismOrientationOptions::v1(), || {
            polls += 1;
            if polls == 2 {
                edit_target(0, &fixture.shapes, &fixture.receipt);
            }
            false
        })
        .ok()
        .unwrap();
    assert_eq!(polls, 2);
    assert_eq!(result.replay_bytes(), before);
    assert!(fixture.rebind(&join).is_err());
}
