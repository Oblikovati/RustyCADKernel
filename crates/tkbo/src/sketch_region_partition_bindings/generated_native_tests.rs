// SPDX-License-Identifier: GPL-2.0-only
//! Replay actual directed builder members; strict candidate failures remain separate.
use super::{corpus::partition_binding_sources, corpus_tests::inspect_corpus_sources};
use crate::sketch_region_candidates::{CandidateCopyInspection, GeneratedTopologyObservation};

#[test]
fn generated_internal_and_closed_members_replay_seven_native_builder_stages() {
    let mut mode = 0;
    let mut expected = Vec::new();
    for line in include_str!("../../tests/fixtures/partition-copy-binding-native.txt").lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields[0] == "mode" {
            if !expected.is_empty() {
                assert_native_members(mode, &expected);
            }
            mode = fields[1].parse().unwrap();
            expected.clear();
        }
        if fields[0] == "member" {
            expected.push(native_member(&fields));
        }
    }
    assert_native_members(mode, &expected);
}
fn native_member(fields: &[&str]) -> [usize; 4] {
    assert_eq!(fields.len(), 5);
    std::array::from_fn(|index| fields[index + 1].parse().unwrap())
}
fn assert_native_members(mode: usize, expected: &[[usize; 4]]) {
    let outputs = inspect_corpus_sources(&partition_binding_sources(mode));
    let copied = outputs.copied.as_ref().unwrap();
    assert_eq!(observed_members(copied), expected, "native mode={mode}");
    assert_eq!(copied.completion.is_ok(), mode > 1);
    assert!(copied.observation_failures.is_empty());
}
fn observed_members(copied: &CandidateCopyInspection) -> Vec<[usize; 4]> {
    let GeneratedTopologyObservation::Observed(areas) = &copied.generated_topology else {
        panic!("expected independent generated topology observations");
    };
    areas
        .iter()
        .flat_map(|area| {
            area.members.iter().map(|member| {
                [
                    area.ordinal,
                    member.kind as usize,
                    member.occurrence.orientation as usize,
                    copied_index(&member.copy_indices),
                ]
            })
        })
        .collect()
}
fn copied_index(indices: &[usize]) -> usize {
    assert!(
        indices.len() <= 1,
        "indices={indices:?}; expected physical copy uniqueness"
    );
    indices.first().copied().unwrap_or(0)
}
