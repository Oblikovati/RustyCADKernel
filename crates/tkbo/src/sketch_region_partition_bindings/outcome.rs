// SPDX-License-Identifier: GPL-2.0-only
//! Native indices/parameters remain observations even when their bindings are conclusive.
pub(crate) struct PartitionBindingInspection {
    pub endpoints: Vec<PartitionEndpointBinding>,
    pub native_spans: Vec<SourceSpanBinding>,
    pub image_span_faults: Vec<super::image_partitions::PartitionImageSpanFault>,
    pub structural_faults: Vec<super::integrity::PartitionStructuralFault>,
    pub canonical_contact: CanonicalContactGap,
}
pub(crate) struct PartitionEndpointBinding {
    pub vertex_index: i32,
    pub incidences: Vec<PartitionEndpointIncidence>,
}
pub(crate) struct PartitionEndpointIncidence {
    pub source_index: i32,
    pub origin: EndpointIncidenceOrigin,
    pub own_edge: i32,
    pub representative: Option<RepresentativeEndpointBinding>,
    pub parameter: f64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalContactGap {
    MissingPairedSourceProof,
}

pub(crate) struct SourceSpanBinding {
    pub source_index: i32,
    pub observation: Result<super::spans::NativeSpanAudit, super::spans::NativeSpanCause>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EndpointIncidenceOrigin {
    OwnPaveBoundary,
    UnreferencedSourceChild,
}

pub(crate) struct RepresentativeEndpointBinding {
    pub source_index: i32,
    pub edge_index: i32,
    pub parameters_at_vertex: Vec<f64>,
}
