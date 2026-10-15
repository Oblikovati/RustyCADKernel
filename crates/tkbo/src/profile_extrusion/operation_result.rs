// SPDX-License-Identifier: GPL-2.0-only
//! `OperationResult`: the immutable, owned B-rep result a feature evaluator consumes: the
//! solid, complete source to result history, validation, findings and achieved tolerance.
use super::profile::{ProfileCurveId, ProfileVertexId};
use super::regions::RegionId;
use super::resolution::{AchievedTolerance, Measure, Resolution};
use super::validation::{Finding, ValidationReport};
use rustycad_tkbrep::topods::{Edge, Face, Shape, Solid, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};

/// An input entity that history starts from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HistorySource {
    /// The swept planar region (its face).
    Region(RegionId),
    /// A caller curve (its single edge in the region).
    Curve(ProfileCurveId),
    /// A caller vertex of the region boundary.
    Vertex(ProfileVertexId),
}

/// A result entity: an index into `OperationResult::faces/edges/vertices`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResultEntity {
    Face(usize),
    Edge(usize),
    Vertex(usize),
}

/// The role a result entity plays relative to its source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HistoryRole {
    /// Face at the region's position.
    StartCap,
    /// Face at the region's position plus the extrusion vector.
    EndCap,
    /// Lateral face swept by a curve.
    SideFace,
    /// Copy of a curve in the start cap.
    StartEdge,
    /// Copy of a curve in the end cap.
    EndEdge,
    /// Lateral edge swept by a vertex (for a full circle: by its seam vertex).
    LateralEdge,
    /// Copy of a vertex in the start cap.
    StartVertex,
    /// Copy of a vertex in the end cap.
    EndVertex,
}

/// The relation class of one history record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HistoryRelation {
    /// The result entity is new and created from the source.
    Generated,
    /// The result entity is the source, changed in place.
    Modified,
}

/// One source to result relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HistoryRecord {
    pub source: HistorySource,
    pub role: HistoryRole,
    pub relation: HistoryRelation,
    pub result: ResultEntity,
}

/// Complete history: every result entity has exactly one record and every source at least
/// one; `deleted` lists sources with no image. Records are sorted, so equal inputs give
/// byte-identical histories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationHistory {
    pub records: Vec<HistoryRecord>,
    pub deleted: Vec<HistorySource>,
}

impl OperationHistory {
    /// Records of `source`, in role order; `history.of(HistorySource::Curve(id))`.
    pub fn of(&self, source: HistorySource) -> Vec<HistoryRecord> {
        self.records
            .iter()
            .filter(|record| record.source == source)
            .copied()
            .collect()
    }

    /// The record whose result is `entity`, if any; `history.parent(ResultEntity::Face(0))`.
    pub fn parent(&self, entity: ResultEntity) -> Option<HistoryRecord> {
        self.records
            .iter()
            .find(|record| record.result == entity)
            .copied()
    }
}

/// Immutable owned result of a modelling operation.
///
/// The solid's topology is indexed once, in the deterministic native traversal order; the
/// index is what `ResultEntity` refers to. Readers get shared references; a caller that
/// needs to modify geometry takes `detached_solid()`, a deep copy with fresh geometry.
#[derive(Clone, Debug)]
pub struct OperationResult {
    pub(crate) solid: Solid,
    pub(crate) faces: Vec<Face>,
    pub(crate) edges: Vec<Edge>,
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) history: OperationHistory,
    pub(crate) validation: ValidationReport,
    pub(crate) findings: Vec<Finding>,
    pub(crate) achieved: AchievedTolerance,
    pub(crate) volume: Measure,
    pub(crate) resolution: Resolution,
}

impl OperationResult {
    /// The solid, read-only; `result.solid()`.
    pub fn solid(&self) -> &Solid {
        &self.solid
    }
    /// A deep copy of the solid sharing no topology or geometry; `result.detached_solid()?`.
    pub fn detached_solid(&self) -> Result<Shape, Failure> {
        let mut copier = BRepBuilderAPICopy::with_shape(&self.solid, true, true)?;
        Ok(copier.shape()?.clone())
    }
    /// Indexed faces; `result.faces()[i]` is `ResultEntity::Face(i)`.
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }
    /// Indexed edges.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }
    /// Indexed vertices.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    /// Complete source to result history.
    pub fn history(&self) -> &OperationHistory {
        &self.history
    }
    /// Ordered validation of the solid.
    pub fn validation(&self) -> ValidationReport {
        self.validation
    }
    /// Non-fatal findings, such as a tolerance above a report limit.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
    /// Largest stored tolerances of the solid.
    pub fn achieved_tolerance(&self) -> AchievedTolerance {
        self.achieved
    }
    /// Volume of the solid with its error and evidence kind.
    pub fn volume(&self) -> Measure {
        self.volume
    }
    /// The resolution the operation ran with.
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }
}
