// SPDX-License-Identifier: GPL-2.0-only
use super::outcome::*;
use crate::sketch_region::{OwnedPartitionSnapshot, SourcePartitionObservation};
/// Attribute actual observed endpoints; `inspect_partition_bindings(&snapshot)`.
/// This stage never substitutes native incidence for paired algebraic contact proof.
pub(crate) fn inspect_partition_bindings(
    snapshot: &OwnedPartitionSnapshot,
) -> PartitionBindingInspection {
    let endpoints = snapshot
        .vertices
        .iter()
        .map(|vertex| PartitionEndpointBinding {
            vertex_index: vertex.shape_index,
            incidences: endpoint_incidences(vertex.shape_index, &snapshot.sources),
        })
        .collect();
    PartitionBindingInspection {
        endpoints,
        native_spans: source_span_bindings(&snapshot.sources),
        structural_faults: super::integrity::inspect_snapshot_integrity(snapshot),
        canonical_contact: CanonicalContactGap::MissingPairedSourceProof,
    }
}
fn endpoint_incidences(
    vertex: i32,
    sources: &[SourcePartitionObservation],
) -> Vec<PartitionEndpointIncidence> {
    sources
        .iter()
        .flat_map(|source| source_incidences(vertex, source))
        .collect()
}
fn source_incidences(
    vertex: i32,
    source: &SourcePartitionObservation,
) -> Vec<PartitionEndpointIncidence> {
    let mut incidences: Vec<_> = source
        .blocks
        .iter()
        .flat_map(|block| pave_incidences(vertex, source.source_index, block))
        .collect();
    if !source.has_reference {
        incidences.extend(unreferenced_incidences(vertex, source));
    }
    incidences
}
fn pave_incidences(
    vertex: i32,
    source: i32,
    block: &crate::sketch_region::PavePartitionObservation,
) -> Vec<PartitionEndpointIncidence> {
    (0..2)
        .filter_map(|end| {
            (block.own_vertices[end] == vertex).then_some(PartitionEndpointIncidence {
                source_index: source,
                origin: EndpointIncidenceOrigin::OwnPaveBoundary,
                own_edge: block.own_edge,
                representative: Some(representative_endpoint(vertex, block)),
                parameter: block.own_range[end],
            })
        })
        .collect()
}

fn source_span_bindings(sources: &[SourcePartitionObservation]) -> Vec<SourceSpanBinding> {
    sources
        .iter()
        .map(|source| SourceSpanBinding {
            source_index: source.source_index,
            observation: super::spans::inspect_native_spans(source),
        })
        .collect()
}

fn unreferenced_incidences(
    vertex: i32,
    source: &SourcePartitionObservation,
) -> Vec<PartitionEndpointIncidence> {
    source
        .source_vertices
        .iter()
        .filter(|child| child.shape_index == vertex)
        .map(|child| PartitionEndpointIncidence {
            source_index: source.source_index,
            origin: EndpointIncidenceOrigin::UnreferencedSourceChild,
            own_edge: source.source_index,
            representative: None,
            parameter: child.parameter,
        })
        .collect()
}

fn representative_endpoint(
    vertex: i32,
    block: &crate::sketch_region::PavePartitionObservation,
) -> RepresentativeEndpointBinding {
    let parameters_at_vertex = (0..2)
        .filter(|&end| block.real_vertices[end] == vertex)
        .map(|end| block.real_range[end])
        .collect();
    RepresentativeEndpointBinding {
        source_index: block.real_original_edge,
        edge_index: block.real_edge,
        parameters_at_vertex,
    }
}
