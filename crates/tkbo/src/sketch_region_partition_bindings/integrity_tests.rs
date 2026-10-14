// SPDX-License-Identifier: GPL-2.0-only
//! Mutation here targets owned receipts, never mapped inputs or numeric policy.
use super::{fixtures::*, integrity::*};

#[test]
fn real_own_and_representative_endpoints_have_single_ds_rows() {
    let snapshot = actual_binding_snapshot(&two_root_edges());
    assert!(inspect_snapshot_integrity(&snapshot).is_empty());
}
#[test]
fn omitted_vertex_observation_retains_affected_source_and_index() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    let omitted = snapshot.vertices.remove(0).shape_index;
    let faults = inspect_snapshot_integrity(&snapshot);
    assert!(faults.iter().any(|fault| fault.source_index == Some(0)
        && fault.cause
            == PartitionStructuralCause::VertexMultiplicity {
                vertex: omitted,
                count: 0,
            }));
}
#[test]
fn duplicate_ds_index_is_ambiguous_even_when_both_points_are_finite() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    let duplicate = snapshot.vertices[0].shape_index;
    snapshot.vertices[1].shape_index = duplicate;
    let faults = inspect_snapshot_integrity(&snapshot);
    assert!(
        faults
            .iter()
            .any(|fault| fault.cause == PartitionStructuralCause::DuplicateVertex(duplicate))
    );
}
#[test]
fn foreign_own_original_cannot_be_attributed_from_matching_endpoints() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].blocks[0].original_edge = 3;
    let faults = inspect_snapshot_integrity(&snapshot);
    assert!(faults.iter().any(|fault| fault.source_index == Some(0)
        && fault.cause
            == PartitionStructuralCause::OwnOriginalMismatch {
                expected: 0,
                actual: 3,
            }));
}

#[test]
fn representative_parameters_follow_actual_vertex_indices_not_endpoint_position() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    let block = &mut snapshot.sources[0].blocks[0];
    block.real_vertices.swap(0, 1);
    block.real_range.swap(0, 1);
    block.real_original_edge = 3;
    let inspected = super::inspect_partition_bindings(&snapshot);
    let first = &inspected.endpoints[0].incidences[0];
    let representative = first.representative.as_ref().unwrap();
    assert_eq!(representative.source_index, 3);
    assert_eq!(representative.parameters_at_vertex, vec![first.parameter]);
}
