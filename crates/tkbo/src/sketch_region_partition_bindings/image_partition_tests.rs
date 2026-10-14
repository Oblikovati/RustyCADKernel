// SPDX-License-Identifier: GPL-2.0-only
//! Receipt negatives never modify the actual mapped DS or input model.
use super::{fixtures::*, image_partitions::*};
use crate::sketch_region::PartitionImageObservation;

#[test]
fn actual_two_root_images_match_representative_edge_indices() {
    let snapshot = actual_binding_snapshot(&two_root_edges());
    assert!(inspect_image_partitions(&snapshot).is_empty());
}
#[test]
fn deleting_an_image_cannot_leave_a_representative_span_unbound() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].images.as_mut().unwrap().remove(0);
    let faults = inspect_image_partitions(&snapshot);
    assert!(faults.iter().any(|fault| fault.source_index == 0
        && fault.block_position == Some(0)
        && fault.cause == PartitionImageSpanCause::RepresentativeMultiplicity { count: 0 }));
}
#[test]
fn duplicated_image_rows_are_not_disambiguated_by_adjacency() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    let images = snapshot.sources[0].images.as_mut().unwrap();
    images.push(PartitionImageObservation {
        shape_index: images[0].shape_index,
        occurrence: images[0].occurrence.clone(),
    });
    let faults = inspect_image_partitions(&snapshot);
    assert!(faults.iter().any(
        |fault| fault.cause == PartitionImageSpanCause::RepresentativeMultiplicity { count: 2 }
    ));
}
#[test]
fn foreign_image_index_is_not_recovered_from_shared_original_parents() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].images.as_mut().unwrap()[0].shape_index = i32::MAX;
    let faults = inspect_image_partitions(&snapshot);
    assert!(
        faults.iter().any(|fault| fault.edge_index == i32::MAX
            && fault.cause == PartitionImageSpanCause::ExtraImage)
    );
}
