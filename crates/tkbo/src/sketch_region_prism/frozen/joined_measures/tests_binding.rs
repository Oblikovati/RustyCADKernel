// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[test]
fn complete_binding_refuses_equal_geometry_with_different_authored_names() {
    let mut fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    let report = measured(&join);
    fixture.authored.operation = AuthoredPrismId(1700);
    let other = fixture.compose(|| false).ok().unwrap();
    assert!(!report.matches_join(&other));
    assert_ne!(report.replay_bytes(), measured(&other).replay_bytes());
    assert!(report.face_area(&other.faces()[0].occurrence).is_none());
}
#[test]
fn authored_binding_permutation_keeps_full_measure_replay() {
    let mut fixture = JoinFixture::new();
    let original = measured(&fixture.compose(|| false).ok().unwrap()).replay_bytes();
    fixture.authored.sources.reverse();
    fixture.authored.directed.reverse();
    fixture.authored.originals.reverse();
    let permuted = measured(&fixture.compose(|| false).ok().unwrap()).replay_bytes();
    assert_eq!(permuted, original);
}
#[test]
fn measurement_retains_every_trim_and_parent_multiplicity_from_the_join() {
    let fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    let report = measured(&join);
    for face in join.faces() {
        let named = report.face_area(&face.occurrence).unwrap();
        assert_eq!(named.face, face);
    }
    assert_eq!(
        report
            .named_faces()
            .iter()
            .map(|face| face.trims.len())
            .sum::<usize>(),
        6
    );
}
#[test]
fn disposed_mutable_owners_cannot_change_measurement_or_replay() {
    let fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    let report = measured(&join);
    let expected = report.replay_bytes();
    drop(fixture);
    assert_eq!(measured(&join).replay_bytes(), expected);
    drop(join);
    assert_eq!(report.replay_bytes(), expected);
}
#[test]
fn changed_source_during_final_callback_requires_separate_source_rebind() {
    use crate::sketch_region_prism::region_certificate::measurements::tests::mutation_tests::edit_target;
    let fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    let (polls, expected) = measured_with_polls(&join);
    let mut current = 0;
    let report = inspect_joined_prism_measures(&join, options(), || {
        current += 1;
        if current == polls {
            edit_target(0, &fixture.shapes, &fixture.receipt);
        }
        false
    })
    .unwrap();
    assert_eq!(report.replay_bytes(), expected);
    assert!(fixture.rebind(&join).is_err());
}

fn measured_with_polls(join: &JoinedPrismProof) -> (usize, Vec<u8>) {
    let mut polls = 0;
    let report = inspect_joined_prism_measures(join, options(), || {
        polls += 1;
        false
    })
    .unwrap();
    (polls, report.replay_bytes())
}
