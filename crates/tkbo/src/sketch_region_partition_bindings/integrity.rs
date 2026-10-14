// SPDX-License-Identifier: GPL-2.0-only
//! Reject missing/ambiguous DS observations before deriving endpoint bindings.
use crate::sketch_region::{OwnedPartitionSnapshot, SourcePartitionObservation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PartitionStructuralFault {
    pub source_index: Option<i32>,
    pub cause: PartitionStructuralCause,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PartitionStructuralCause {
    DuplicateVertex(i32),
    VertexMultiplicity { vertex: i32, count: usize },
    OwnOriginalMismatch { expected: i32, actual: i32 },
}
pub(super) fn inspect_snapshot_integrity(
    snapshot: &OwnedPartitionSnapshot,
) -> Vec<PartitionStructuralFault> {
    let mut faults = duplicate_vertices(snapshot);
    for source in &snapshot.sources {
        inspect_source_vertices(snapshot, source, &mut faults);
    }
    faults
}
fn duplicate_vertices(snapshot: &OwnedPartitionSnapshot) -> Vec<PartitionStructuralFault> {
    let mut seen = std::collections::BTreeSet::new();
    snapshot
        .vertices
        .iter()
        .filter_map(|vertex| {
            (!seen.insert(vertex.shape_index)).then_some(PartitionStructuralFault {
                source_index: None,
                cause: PartitionStructuralCause::DuplicateVertex(vertex.shape_index),
            })
        })
        .collect()
}
fn inspect_source_vertices(
    snapshot: &OwnedPartitionSnapshot,
    source: &SourcePartitionObservation,
    faults: &mut Vec<PartitionStructuralFault>,
) {
    for child in &source.source_vertices {
        check_vertex(snapshot, source.source_index, child.shape_index, faults);
    }
    for block in &source.blocks {
        check_original(source.source_index, block.original_edge, faults);
        for vertex in block.own_vertices.into_iter().chain(block.real_vertices) {
            check_vertex(snapshot, source.source_index, vertex, faults);
        }
    }
}
fn check_original(expected: i32, actual: i32, faults: &mut Vec<PartitionStructuralFault>) {
    if expected == actual {
        return;
    }
    faults.push(PartitionStructuralFault {
        source_index: Some(expected),
        cause: PartitionStructuralCause::OwnOriginalMismatch { expected, actual },
    });
}
fn check_vertex(
    snapshot: &OwnedPartitionSnapshot,
    source: i32,
    vertex: i32,
    faults: &mut Vec<PartitionStructuralFault>,
) {
    let count = snapshot
        .vertices
        .iter()
        .filter(|row| row.shape_index == vertex)
        .count();
    if count == 1 {
        return;
    }
    faults.push(PartitionStructuralFault {
        source_index: Some(source),
        cause: PartitionStructuralCause::VertexMultiplicity { vertex, count },
    });
}
