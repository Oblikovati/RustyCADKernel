// SPDX-License-Identifier: GPL-2.0-only
//! Receipt-policy tests do not stand in for actual copier/native provenance tests.
use super::{fixtures::*, image_links::*};
use crate::sketch_region::*;
use crate::sketch_region_candidates::CandidateImageCopyLink;

struct RecordedCopyRows;
impl RecordedCopyRows {
    fn from_snapshot(snapshot: &OwnedPartitionSnapshot) -> Vec<CandidateImageCopyLink> {
        snapshot
            .sources
            .iter()
            .flat_map(|source| {
                source
                    .images
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|image| recorded_row(source, image))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

#[test]
fn every_observed_image_requires_a_physical_copy_row() {
    let snapshot = actual_binding_snapshot(&two_root_edges());
    let bindings = inspect_image_links(&snapshot, &[]);
    assert_eq!(bindings.len(), 6);
    assert!(
        bindings
            .iter()
            .all(|binding| binding.fault == Some(PartitionImageCause::MissingCopy))
    );
}
#[test]
fn equal_parent_sets_do_not_recover_a_missing_original_image() {
    let snapshot = actual_binding_snapshot(&two_root_edges());
    let mut rows = RecordedCopyRows::from_snapshot(&snapshot);
    rows[0].original.topology_address = usize::MAX;
    let bindings = inspect_image_links(&snapshot, &rows);
    assert_eq!(bindings[0].fault, Some(PartitionImageCause::MissingCopy));
}
#[test]
fn copied_image_parent_omission_is_attributed_to_the_actual_row() {
    let snapshot = actual_binding_snapshot(&two_root_edges());
    let mut rows = RecordedCopyRows::from_snapshot(&snapshot);
    rows[0].modified.clear();
    let bindings = inspect_image_links(&snapshot, &rows);
    assert_eq!(
        bindings[0].fault,
        Some(PartitionImageCause::MissingParent {
            copy_row: 0,
            parent_position: 0
        })
    );
}
#[test]
fn physical_builder_image_membership_preserves_directed_copy_aliases() {
    let snapshot = actual_binding_snapshot(&two_root_edges());
    let mut rows = RecordedCopyRows::from_snapshot(&snapshot);
    rows[0].original.orientation = rustycad_tkg3d::top_abs::Orientation::Reversed;
    let bindings = inspect_image_links(&snapshot, &rows);
    assert_eq!(bindings[0].copy_rows, vec![0]);
    assert!(bindings.iter().all(|binding| binding.fault.is_none()));
}

fn recorded_row(
    source: &SourcePartitionObservation,
    image: &PartitionImageObservation,
) -> CandidateImageCopyLink {
    CandidateImageCopyLink {
        original: image.occurrence.clone(),
        kind: Ok(rustycad_tkg3d::top_abs::ShapeEnum::Edge),
        history_supported: Ok(true),
        copied: image.occurrence.clone(),
        preserved: Vec::new(),
        modified: source.parents.clone(),
        generated: Vec::new(),
    }
}

#[test]
fn missing_split_image_context_is_not_an_unsplit_identity_copy() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].images = None;
    let bindings = inspect_image_links(&snapshot, &[]);
    assert_eq!(
        bindings[0].fault,
        Some(PartitionImageCause::MissingImageContext)
    );
}
#[test]
fn bound_empty_image_list_is_not_completed_empty_candidate_coverage() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].images = Some(Vec::new());
    let bindings = inspect_image_links(&snapshot, &[]);
    assert_eq!(bindings[0].fault, Some(PartitionImageCause::EmptyImageMap));
}
