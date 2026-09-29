// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_RelatedIterator.hxx and
// .cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_RelatedIterator`: the semantically related nodes of one face, edge, wire,
//! vertex or coedge, with the relation that relates each; results are not deduplicated.
//!
//! The iterator resumes each relation from saved indices (`myIndex`, `myInnerIndex`,
//! `myDeepIndex`) as natively. A relation whose next node is inactive ends that relation
//! (the native `setCurrent` returns false and the stage moves on). The face's outer
//! wire comes from `BRepGraph_Tool::Face::OuterWire`, whose p-curve adaptors may raise,
//! so the constructor and `next` return `Result`.
use crate::brep_graph::defs_iterator::{
    DefsCoEdgeOfWire, DefsEdgeOfWire, DefsVertexOfEdge, DefsWireOfFace,
};
use crate::brep_graph::refs_iterator::CoEdgesOfWire;
use crate::brep_graph::reverse_iterator::{EdgesOfVertex, FacesOfWire};
use crate::brep_graph::tool;
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId, NodeId, NodeKind, VertexId, WireId};
use rustycad_tkernel::standard::Failure;

/// `BRepGraph_RelatedIterator::RelationKind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RelationKind {
    /// Face to an edge bounding it.
    #[default]
    BoundaryEdge,
    /// Face to a face sharing an edge.
    AdjacentFace,
    /// Face to its outer wire.
    OuterWire,
    /// Edge to a face that uses it.
    ReferencedByFace,
    /// Edge to its start or end vertex.
    IncidentVertex,
    /// Wire to a member coedge.
    WireCoEdge,
    /// Wire or coedge to its face.
    OwningFace,
    /// Vertex to a touching edge.
    IncidentEdge,
    /// Coedge to its edge.
    ParentEdge,
    /// Coedge to its seam twin.
    SeamPair,
}

/// `BRepGraph_RelatedIterator::Stage`: the active sub-iteration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Stage {
    /// The primary relation.
    #[default]
    First,
    /// The secondary relation.
    Second,
    /// The tertiary relation.
    Third,
    /// All relations exhausted.
    Finished,
}

/// `BRepGraph_RelatedIterator`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::related_iterator::RelatedIterator;
/// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceId};
/// let graph = BRepGraph::new();
/// assert!(!RelatedIterator::new(&graph, FaceId::new(0).into()).unwrap().more());
/// ```
pub struct RelatedIterator<'g> {
    graph: &'g BRepGraph,
    node: NodeId,
    current: NodeId,
    relation: RelationKind,
    stage: Stage,
    index: u32,
    inner_index: u32,
    deep_index: u32,
    has_current: bool,
}

impl<'g> RelatedIterator<'g> {
    /// `BRepGraph_RelatedIterator(theGraph, theNode)`: positioned on the first related
    /// node.
    pub fn new(graph: &'g BRepGraph, node: NodeId) -> Result<Self, Failure> {
        let mut iterator = Self {
            graph,
            node,
            current: NodeId::default(),
            relation: RelationKind::BoundaryEdge,
            stage: Stage::First,
            index: 0,
            inner_index: 0,
            deep_index: 0,
            has_current: false,
        };
        iterator.advance()?;
        Ok(iterator)
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.has_current
    }

    /// `Next()`: nothing once exhausted.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible; Iterator is implemented too.
    pub fn next(&mut self) -> Result<(), Failure> {
        if !self.has_current {
            return Ok(());
        }
        self.advance()
    }

    /// `Current()`: the current related node (the last one after the end).
    pub fn current(&self) -> NodeId {
        self.current
    }

    /// `CurrentRelation()`.
    pub fn current_relation(&self) -> RelationKind {
        self.relation
    }

    // `setCurrent`: accepts a valid, active node.
    fn set_current(&mut self, node: NodeId, relation: RelationKind) -> bool {
        if !node.is_valid() || self.graph.topo().generic().is_removed(node) {
            return false;
        }
        self.current = node;
        self.relation = relation;
        self.has_current = true;
        true
    }

    fn advance(&mut self) -> Result<(), Failure> {
        self.has_current = false;
        if !self.node.is_valid() || self.graph.topo().generic().is_removed(self.node) {
            return Ok(());
        }
        match self.node.kind {
            NodeKind::FACE => self.advance_face()?,
            NodeKind::EDGE => self.advance_edge(),
            NodeKind::WIRE => self.advance_wire(),
            NodeKind::VERTEX => {
                self.advance_incident_edge();
            }
            NodeKind::CO_EDGE => self.advance_co_edge(),
            // Container and assembly nodes have no topological relations.
            _ => {}
        }
        Ok(())
    }

    fn advance_face(&mut self) -> Result<(), Failure> {
        if self.stage == Stage::First {
            if self.advance_face_boundary_edge() {
                return Ok(());
            }
            self.stage = Stage::Second;
            (self.index, self.inner_index, self.deep_index) = (0, 0, 0);
        }
        if self.stage == Stage::Second {
            if self.advance_adjacent_face() {
                return Ok(());
            }
            self.stage = Stage::Third;
            self.index = 0;
        }
        if self.stage == Stage::Third {
            self.stage = Stage::Finished;
            let outer = tool::face::outer_wire(self.graph, FaceId::from_node_id(self.node))?;
            self.set_current(outer.into(), RelationKind::OuterWire);
        }
        Ok(())
    }

    // `advanceFaceBoundaryEdge`: the edges of the face's wires, resumed at
    // (myIndex, myInnerIndex).
    fn advance_face_boundary_edge(&mut self) -> bool {
        let face = FaceId::from_node_id(self.node);
        let mut wires = DefsWireOfFace::new(self.graph, face);
        while wires.more() {
            let (wire_index, wire) = (wires.index(), wires.current_id());
            wires.next();
            if wire_index < self.index {
                continue;
            }
            let mut edges = DefsEdgeOfWire::new(self.graph, wire);
            while edges.more() {
                if wire_index == self.index && edges.index() < self.inner_index {
                    edges.next();
                    continue;
                }
                self.index = wire_index;
                self.inner_index = edges.index() + 1;
                return self.set_current(edges.current_id().into(), RelationKind::BoundaryEdge);
            }
            self.index = wire_index + 1;
            self.inner_index = 0;
        }
        false
    }

    // `advanceAdjacentFace`: the other faces of the edges of the face's coedges,
    // resumed at (myIndex, myInnerIndex, myDeepIndex).
    fn advance_adjacent_face(&mut self) -> bool {
        let face = FaceId::from_node_id(self.node);
        let mut wires = DefsWireOfFace::new(self.graph, face);
        while wires.more() {
            let (wire_index, wire) = (wires.index(), wires.current_id());
            wires.next();
            if wire_index < self.index {
                continue;
            }
            if self.adjacent_through_wire(face, wire_index, wire) {
                return true;
            }
            self.index = wire_index + 1;
            self.inner_index = 0;
        }
        false
    }

    // The coedge loop of `advanceAdjacentFace` for one wire.
    fn adjacent_through_wire(&mut self, face: FaceId, wire_index: u32, wire: WireId) -> bool {
        let mut co_edges = DefsCoEdgeOfWire::new(self.graph, wire);
        while co_edges.more() {
            let (co_edge_index, edge) = (co_edges.index(), co_edges.current().child_edge_id);
            co_edges.next();
            if wire_index == self.index && co_edge_index < self.inner_index {
                continue;
            }
            if let Some(found) = self.adjacent_through_edge(face, edge) {
                self.index = wire_index;
                self.inner_index = co_edge_index;
                return self.set_current(found.into(), RelationKind::AdjacentFace);
            }
            self.deep_index = 0;
        }
        false
    }

    // The face loop of `advanceAdjacentFace` for one edge, from myDeepIndex; the found
    // face's index + 1 becomes the next myDeepIndex.
    fn adjacent_through_edge(&mut self, face: FaceId, edge: EdgeId) -> Option<FaceId> {
        let mut faces = self
            .graph
            .topo()
            .edges()
            .faces_of_from(edge, self.deep_index);
        while faces.more() {
            let adjacent = faces.current_id();
            self.deep_index = faces.index() + 1;
            if adjacent != face {
                return Some(adjacent);
            }
            faces.next();
        }
        None
    }

    fn advance_edge(&mut self) {
        let edge = EdgeId::from_node_id(self.node);
        if self.stage == Stage::First {
            let faces = self.graph.topo().edges().faces_of_from(edge, self.index);
            if faces.more() {
                self.index = faces.index() + 1;
                if self.set_current(faces.current_id().into(), RelationKind::ReferencedByFace) {
                    return;
                }
            }
            self.stage = Stage::Second;
            self.index = 0;
        }
        // `advanceEdgeVertex` through `advanceDefChildren`.
        let mut vertices = DefsVertexOfEdge::new(self.graph, edge);
        while vertices.more() {
            if vertices.index() < self.index {
                vertices.next();
                continue;
            }
            self.index = vertices.index() + 1;
            let vertex: VertexId = vertices.current_id();
            self.set_current(vertex.into(), RelationKind::IncidentVertex);
            return;
        }
    }

    fn advance_wire(&mut self) {
        let wire = WireId::from_node_id(self.node);
        if self.stage == Stage::First {
            // `advanceRefChildren` over the wire's coedges.
            let mut co_edges = CoEdgesOfWire::new(self.graph, wire);
            while co_edges.more() && co_edges.index() < self.index {
                co_edges.next();
            }
            if co_edges.more() {
                self.index = co_edges.index() + 1;
                let co_edge: CoEdgeId = co_edges.current_id();
                if self.set_current(co_edge.into(), RelationKind::WireCoEdge) {
                    return;
                }
            }
            self.stage = Stage::Second;
            self.index = 0;
        }
        let refs = &self
            .graph
            .topo()
            .wires()
            .relations(wire)
            .parent_wire_ref_ids;
        let faces = FacesOfWire::starting_at(self.graph, refs, self.index);
        if faces.more() {
            self.index = faces.index() + 1;
            self.set_current(faces.current_id().into(), RelationKind::OwningFace);
        }
    }

    // `advanceParents` over the vertex's edges, resumed at myIndex.
    fn advance_incident_edge(&mut self) -> bool {
        let vertex = VertexId::from_node_id(self.node);
        let edges = EdgesOfVertex::starting_at(
            self.graph,
            self.graph.topo().vertices().edges(vertex),
            self.index,
        );
        if !edges.more() {
            return false;
        }
        self.index = edges.index() + 1;
        self.set_current(edges.current_id().into(), RelationKind::IncidentEdge)
    }

    fn advance_co_edge(&mut self) {
        let co_edge = CoEdgeId::from_node_id(self.node);
        if self.stage == Stage::First {
            self.stage = Stage::Second;
            let edge = tool::co_edge::edge_of(self.graph, co_edge);
            if self.set_current(edge.into(), RelationKind::ParentEdge) {
                return;
            }
        }
        if self.stage == Stage::Second {
            self.stage = Stage::Third;
            let face = tool::co_edge::face_of(self.graph, co_edge);
            if self.set_current(face.into(), RelationKind::OwningFace) {
                return;
            }
        }
        if self.stage == Stage::Third {
            self.stage = Stage::Finished;
            let twin = tool::co_edge::seam_pair(self.graph, co_edge);
            self.set_current(twin.into(), RelationKind::SeamPair);
        }
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` node; an error of `next`
/// is yielded once and ends the iteration.
impl Iterator for RelatedIterator<'_> {
    type Item = Result<NodeId, Failure>;
    fn next(&mut self) -> Option<Result<NodeId, Failure>> {
        if !self.more() {
            return None;
        }
        let current = self.current;
        match RelatedIterator::next(self) {
            Ok(()) => Some(Ok(current)),
            Err(error) => {
                self.has_current = false;
                Some(Err(error))
            }
        }
    }
}
