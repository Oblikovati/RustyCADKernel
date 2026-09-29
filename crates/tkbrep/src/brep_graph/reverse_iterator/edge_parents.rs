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
// Rust translation of BRepGraph_ReverseIterator::EdgeParentsOf, WireFromEdgeCoEdgeTraits,
// FaceFromEdgeCoEdgeTraits, BRepGraph_WiresOfEdge and BRepGraph_FacesOfEdge
// (ModelingData/TKBRep/BRepGraph/BRepGraph_ReverseIterator.hxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parents of an edge derived from its coedges.
use crate::brep_graph::iterator::NodeTraits;
use crate::brep_graph::{
    BRepGraph, CoEdgeId, CoEdgeTag, EdgeId, EdgeTag, FaceTag, TypedId, WireTag,
};
use crate::brep_graph_inc::{CoEdgeDef, NodeTag};

/// The traits of [`EdgeParentsOf`]: the parent a coedge names and the parent count.
pub trait EdgeParentTraits {
    /// `ParentId`.
    type Parent: NodeTag;
    /// `ParentIdOf(theCoEdge)`.
    fn parent_id_of(co_edge: &CoEdgeDef) -> TypedId<Self::Parent>;
    /// `NbParents(theGraph)`.
    fn nb_parents(graph: &BRepGraph) -> u32 {
        Self::Parent::count(graph)
    }
}

/// `WireFromEdgeCoEdgeTraits`: the wire of each coedge.
#[derive(Clone, Copy, Debug, Default)]
pub struct WireFromEdgeCoEdgeTraits;

impl EdgeParentTraits for WireFromEdgeCoEdgeTraits {
    type Parent = WireTag;
    fn parent_id_of(co_edge: &CoEdgeDef) -> TypedId<WireTag> {
        co_edge.parent_wire_id
    }
}

/// `FaceFromEdgeCoEdgeTraits`: the face of each coedge.
#[derive(Clone, Copy, Debug, Default)]
pub struct FaceFromEdgeCoEdgeTraits;

impl EdgeParentTraits for FaceFromEdgeCoEdgeTraits {
    type Parent = FaceTag;
    fn parent_id_of(co_edge: &CoEdgeDef) -> TypedId<FaceTag> {
        co_edge.face_id
    }
}

/// `EdgeParentsOf<TraitsT>`: the distinct active parents of an edge's active coedges,
/// each reported at its first coedge.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId};
/// use rustycad_tkbrep::brep_graph::reverse_iterator::FacesOfEdge;
/// assert!(!FacesOfEdge::new(&BRepGraph::new(), EdgeId::new(0)).more());
/// ```
pub struct EdgeParentsOf<'g, P: EdgeParentTraits> {
    graph: &'g BRepGraph,
    co_edges: Option<&'g [CoEdgeId]>,
    current: TypedId<P::Parent>,
    nb_parents: u32,
    index: u32,
    has_current: bool,
}

impl<'g, P: EdgeParentTraits> EdgeParentsOf<'g, P> {
    /// `EdgeParentsOf(theGraph, theEdge)`.
    pub fn new(graph: &'g BRepGraph, edge: EdgeId) -> Self {
        Self::starting_at(graph, edge, 0)
    }

    /// `EdgeParentsOf(theGraph, theEdge, theStartIndex)`: resumes at a coedge index of
    /// the edge's relation list.
    pub fn starting_at(graph: &'g BRepGraph, edge: EdgeId, start_index: u32) -> Self {
        let mut iterator = Self {
            graph,
            co_edges: None,
            current: TypedId::default(),
            nb_parents: 0,
            index: start_index,
            has_current: false,
        };
        // `init`: an invalid or removed edge has no coedge list.
        if edge.is_valid_for(EdgeTag::count(graph)) && !edge.is_removed(graph) {
            iterator.nb_parents = P::nb_parents(graph);
            iterator.co_edges = Some(graph.topo().edges().co_edges(edge));
        }
        iterator.advance();
        iterator
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.has_current
    }

    /// `Next()`.
    pub fn next(&mut self) {
        self.index += 1;
        self.advance();
    }

    /// `CurrentId()`: the current parent (the previous one after the end).
    pub fn current_id(&self) -> TypedId<P::Parent> {
        self.current
    }

    /// `Current()`: the same as [`current_id`](Self::current_id).
    pub fn current(&self) -> TypedId<P::Parent> {
        self.current
    }

    /// `Definition()`: the current parent's definition.
    pub fn definition(&self) -> &'g <P::Parent as NodeTag>::Def {
        P::Parent::get(self.graph, self.current)
    }

    /// `Index()`: the coedge index of the current parent.
    pub fn index(&self) -> u32 {
        self.index
    }

    // `parentAt`: the active parent of the active coedge at `index`, invalid otherwise.
    fn parent_at(&self, index: u32) -> TypedId<P::Parent> {
        let Some(co_edge) = self.co_edges.and_then(|ids| ids.get(index as usize)) else {
            return TypedId::default();
        };
        if !co_edge.is_valid_for(CoEdgeTag::count(self.graph)) || co_edge.is_removed(self.graph) {
            return TypedId::default();
        }
        let parent = P::parent_id_of(CoEdgeTag::get(self.graph, *co_edge));
        if !parent.is_valid_for(self.nb_parents) || parent.is_removed(self.graph) {
            return TypedId::default();
        }
        parent
    }

    // `isFirstOccurrence`: no earlier coedge names the same parent.
    fn is_first_occurrence(&self, parent: TypedId<P::Parent>, index: u32) -> bool {
        (0..index).all(|earlier| self.parent_at(earlier) != parent)
    }

    fn advance(&mut self) {
        self.has_current = false;
        let Some(co_edges) = self.co_edges else {
            return;
        };
        while (self.index as usize) < co_edges.len() {
            let parent = self.parent_at(self.index);
            if parent.is_valid() && self.is_first_occurrence(parent, self.index) {
                self.current = parent;
                self.has_current = true;
                return;
            }
            self.index += 1;
        }
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` id.
impl<P: EdgeParentTraits> Iterator for EdgeParentsOf<'_, P> {
    type Item = TypedId<P::Parent>;
    fn next(&mut self) -> Option<TypedId<P::Parent>> {
        if !self.more() {
            return None;
        }
        let current = self.current;
        EdgeParentsOf::next(self);
        Some(current)
    }
}

/// `BRepGraph_WiresOfEdge`: edge to parent wires.
pub type WiresOfEdge<'g> = EdgeParentsOf<'g, WireFromEdgeCoEdgeTraits>;
/// `BRepGraph_FacesOfEdge`: edge to parent faces (from the coedge faces).
pub type FacesOfEdge<'g> = EdgeParentsOf<'g, FaceFromEdgeCoEdgeTraits>;
