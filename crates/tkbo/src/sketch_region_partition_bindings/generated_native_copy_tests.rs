// SPDX-License-Identifier: GPL-2.0-only
//! Source/image indices and directed copy records replay actual native Modified calls.
use super::{
    corpus::partition_binding_sources, corpus_tests::inspect_corpus_sources, transaction::*,
};
use crate::sketch_region::{
    OwnedPartitionSnapshot, PartitionObservation, PartitionOccurrenceIdentity,
};
use crate::sketch_region_candidates::{CandidateCopyInspection, CandidateCopyTreeObservation};

#[test]
fn actual_source_image_copy_indices_replay_split_and_unsplit_native_history() {
    let mut mode = 0;
    let mut expected = Vec::new();
    for line in include_str!("../../tests/fixtures/partition-copy-binding-native.txt").lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields[0] == "mode" {
            if !expected.is_empty() {
                assert_native_copies(mode, &expected);
            }
            mode = fields[1].parse().unwrap();
            expected.clear();
        }
        if fields[0] == "copy" {
            expected.push(native_copy(&fields));
        }
    }
    assert_native_copies(mode, &expected);
}
fn native_copy(fields: &[&str]) -> [i64; 7] {
    assert_eq!(fields.len(), 8);
    std::array::from_fn(|index| fields[index + 1].parse().unwrap())
}
fn assert_native_copies(mode: usize, expected: &[[i64; 7]]) {
    let outputs = inspect_corpus_sources(&partition_binding_sources(mode));
    assert_eq!(
        observed_copies(&outputs),
        expected,
        "native copy mode={mode}"
    );
}
fn observed_copies(outputs: &PartitionBoundOutputs) -> Vec<[i64; 7]> {
    let PartitionObservation::Captured(snapshot) = &outputs.arrangement.partition else {
        panic!("expected original DS snapshot");
    };
    let copy = outputs.copied.as_ref().unwrap();
    let mut rows = Vec::new();
    append_source_copies(snapshot, copy, &mut rows);
    rows
}
fn append_source_copies(
    snapshot: &OwnedPartitionSnapshot,
    copy: &CandidateCopyInspection,
    rows: &mut Vec<[i64; 7]>,
) {
    for source in &snapshot.sources {
        append_one_source(source, copy, rows);
    }
}
fn append_image_copies(
    source: i32,
    image: i32,
    key: &PartitionOccurrenceIdentity,
    copy: &CandidateCopyInspection,
    rows: &mut Vec<[i64; 7]>,
) {
    for link in copy.links.iter().filter(|link| link.original == *key) {
        rows.push([
            source.into(),
            image.into(),
            link.original.orientation as i64,
            link.copied.orientation as i64,
            actual_copied_index(copy, &link.copied) as i64,
            i64::from(link.original.same_physical(&link.copied)),
            i64::from(*link.history_supported.as_ref().unwrap()),
        ]);
    }
}
fn actual_copied_index(copy: &CandidateCopyInspection, key: &PartitionOccurrenceIdentity) -> usize {
    let CandidateCopyTreeObservation::Captured(tree) = &copy.copied_tree else {
        panic!("expected actual copier tree");
    };
    let matches: Vec<_> = tree
        .iter()
        .filter(|row| row.occurrence.same_physical(key))
        .collect();
    assert_eq!(matches.len(), 1);
    matches[0].index
}

fn append_one_source(
    source: &crate::sketch_region::SourcePartitionObservation,
    copy: &CandidateCopyInspection,
    rows: &mut Vec<[i64; 7]>,
) {
    if let Some(images) = &source.images {
        append_source_images(source.source_index, images, copy, rows);
        return;
    }
    append_image_copies(
        source.source_index,
        source.source_index,
        &source.occurrence,
        copy,
        rows,
    );
}

fn append_source_images(
    source: i32,
    images: &[crate::sketch_region::PartitionImageObservation],
    copy: &CandidateCopyInspection,
    rows: &mut Vec<[i64; 7]>,
) {
    for image in images {
        append_image_copies(source, image.shape_index, &image.occurrence, copy, rows);
    }
}
