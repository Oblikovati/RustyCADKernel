// SPDX-License-Identifier: GPL-2.0-only
//! Observed native state is not parameter/contact/search certification for ADR0014.
use super::outcome::{ArrangementCompletion, OriginalMemberIdentity};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

pub(crate) struct ObservedPartitionArrangement {
    pub completion: ArrangementCompletion,
    pub partition: PartitionObservation,
}
pub(crate) enum PartitionObservation {
    NotReached,
    Captured(OwnedPartitionSnapshot),
    Incomplete {
        partial: OwnedPartitionSnapshot,
        failure: PartitionReadFailure,
    },
}
pub(crate) struct OwnedPartitionSnapshot {
    pub source_shape_count: Option<i32>,
    pub sources: Vec<SourcePartitionObservation>,
    pub vertices: Vec<PartitionVertexObservation>,
}
pub(crate) struct SourcePartitionObservation {
    pub source_index: i32,
    pub parents: Vec<OriginalMemberIdentity>,
    pub occurrence: PartitionOccurrenceIdentity,
    pub range: [f64; 2],
    pub space: Option<PartitionSpaceRead>,
    pub has_reference: bool,
    pub source_vertices: Vec<SourceVertexObservation>,
    pub blocks: Vec<PavePartitionObservation>,
    pub images: Option<Vec<PartitionImageObservation>>,
}
pub(crate) struct SourceVertexObservation {
    pub shape_index: i32,
    pub occurrence: PartitionOccurrenceIdentity,
    pub parameter: f64,
}
pub(crate) struct PartitionVertexObservation {
    pub shape_index: i32,
    pub occurrence: PartitionOccurrenceIdentity,
    pub point: [f64; 3],
    pub original_parents: Vec<OriginalMemberIdentity>,
}
pub(crate) struct PartitionImageObservation {
    pub shape_index: i32,
    pub occurrence: PartitionOccurrenceIdentity,
}
/// Identity only: no mutable shape, DS, or pave-block handle survives in this receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PartitionOccurrenceIdentity {
    pub topology_address: usize,
    pub location: Location,
    pub orientation: Orientation,
}
#[derive(Clone, Copy)]
pub(crate) struct PavePartitionObservation {
    pub original_edge: i32,
    pub own_edge: i32,
    pub real_original_edge: i32,
    pub real_edge: i32,
    pub own_range: [f64; 2],
    pub real_range: [f64; 2],
    pub own_vertices: [i32; 2],
    pub real_vertices: [i32; 2],
}
pub(crate) struct PartitionReadFailure {
    pub source_index: Option<i32>,
    pub vertex_index: Option<i32>,
    pub block_position: Option<usize>,
    pub parents: Vec<OriginalMemberIdentity>,
    pub cause: PartitionReadCause,
}
pub(crate) enum PartitionReadCause {
    Native(Box<Failure>),
    MissingContext(&'static str),
    MissingSourceParents(i32),
    MissingCopiedSource(usize),
    PoisonedPaveBlock,
    MissingImageIndex(i32),
    MissingVertexIndex(i32),
}

pub(crate) type PartitionSpaceCapture = fn(
    &rustycad_tkbrep::topods::Edge,
) -> Result<
    crate::sketch_region_source_conformance::CapturedSpaceRepresentation,
    crate::sketch_region_geometry_capture::CapturedGeometryCause,
>;

pub(crate) type PartitionSpaceRead = Result<
    crate::sketch_region_source_conformance::CapturedSpaceRepresentation,
    std::sync::Arc<crate::sketch_region_geometry_capture::CapturedGeometryCause>,
>;
