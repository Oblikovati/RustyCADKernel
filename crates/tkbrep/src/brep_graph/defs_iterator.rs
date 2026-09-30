// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_DefsIterator.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_DefsIterator`: the active child definitions of one parent, one level deep,
//! in relation order, without location or orientation accumulation.
//!
//! The iterators skip removed reference entries, invalid child ids and removed child
//! definitions. The traits are in [`parent_traits`](crate::brep_graph::parent_traits).
use crate::brep_graph::parent_cursor::{EdgeVertexCursor, ParentCursor};
use crate::brep_graph::parent_traits::{
    ChildOfCompoundTraits, CoEdgeOfWireTraits, EdgeOfWireTraits, FaceOfShellTraits,
    OccurrenceOfProductTraits, ParentTraits, ShellOfSolidTraits, SolidOfCompSolidTraits,
    WireOfFaceTraits,
};
use crate::brep_graph::{BRepGraph, EdgeId, TypedId, VertexId, VertexRefId};
use crate::brep_graph_inc::VertexDef;

/// `BRepGraph_DefsIterator::DefsOfParent<TraitsT>`: the active children of a parent.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceId};
/// use rustycad_tkbrep::brep_graph::defs_iterator::DefsWireOfFace;
/// let graph = BRepGraph::new();
/// assert!(!DefsWireOfFace::new(&graph, FaceId::new(0)).more());
/// ```
pub struct DefsOfParent<'g, P: ParentTraits> {
    cursor: ParentCursor<'g, P>,
    current_child: P::Child,
}

impl<'g, P: ParentTraits> DefsOfParent<'g, P> {
    /// `DefsOfParent(theGraph, theParent)`: positioned on the first active child; an
    /// invalid or removed parent yields nothing.
    pub fn new(graph: &'g BRepGraph, parent: TypedId<P::Parent>) -> Self {
        let mut iterator = Self {
            cursor: ParentCursor::new(graph, parent),
            current_child: P::Child::default(),
        };
        iterator.skip_removed();
        iterator
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.cursor.more()
    }

    /// `Next()`.
    pub fn next(&mut self) {
        self.cursor.index += 1;
        self.skip_removed();
    }

    /// `CurrentId()`: the current child (past the end, the last child found, as the
    /// release build returns).
    pub fn current_id(&self) -> P::Child {
        self.current_child
    }

    /// `Current()`: the current child definition (panics when no child was found).
    pub fn current(&self) -> &'g P::ChildDef {
        P::child(self.cursor.graph, self.current_child)
    }

    /// `CurrentRefId()`: the reference or coedge carrying the current relation, invalid
    /// past the end.
    pub fn current_ref_id(&self) -> P::Ref {
        self.cursor.ref_at_index().unwrap_or_default()
    }

    /// `Index()`: the position in the parent's relation list.
    pub fn index(&self) -> u32 {
        self.cursor.index
    }

    fn skip_removed(&mut self) {
        if let Some(child) = self.cursor.skip_removed() {
            self.current_child = child;
        }
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` definition.
impl<'g, P: ParentTraits> Iterator for DefsOfParent<'g, P> {
    type Item = &'g P::ChildDef;
    fn next(&mut self) -> Option<&'g P::ChildDef> {
        if !self.more() {
            return None;
        }
        let current = self.current();
        DefsOfParent::next(self);
        Some(current)
    }
}

/// `BRepGraph_DefsIterator::DefsVertexOfEdge`: the active start then end vertex of an
/// edge.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId};
/// use rustycad_tkbrep::brep_graph::defs_iterator::DefsVertexOfEdge;
/// let graph = BRepGraph::new();
/// assert!(!DefsVertexOfEdge::new(&graph, EdgeId::new(0)).more());
/// ```
pub struct DefsVertexOfEdge<'g> {
    cursor: EdgeVertexCursor<'g>,
}

impl<'g> DefsVertexOfEdge<'g> {
    /// `DefsVertexOfEdge(theGraph, theEdgeId)`: an invalid or removed edge yields nothing.
    pub fn new(graph: &'g BRepGraph, edge: EdgeId) -> Self {
        Self {
            cursor: EdgeVertexCursor::new(graph, edge),
        }
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.cursor.more()
    }

    /// `Next()`.
    pub fn next(&mut self) {
        self.cursor.next();
    }

    /// `CurrentId()`: the vertex of the current reference (panics past the end, where
    /// the native release build reads the end reference).
    pub fn current_id(&self) -> VertexId {
        let reference = self.cursor.current_ref_id();
        self.cursor
            .graph
            .inc_storage()
            .vertex_ref(reference)
            .child_vertex_id
    }

    /// `Current()`: the current vertex definition.
    pub fn current(&self) -> &'g VertexDef {
        let graph = self.cursor.graph;
        graph.inc_storage().vertex(self.current_id())
    }

    /// `CurrentRefId()`: the start or end vertex reference, invalid past the end.
    pub fn current_ref_id(&self) -> VertexRefId {
        if !self.more() {
            return VertexRefId::default();
        }
        self.cursor.current_ref_id()
    }

    /// `Index()`: 0 for the start vertex, 1 for the end vertex.
    pub fn index(&self) -> u32 {
        self.cursor.index
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` vertex definition.
impl<'g> Iterator for DefsVertexOfEdge<'g> {
    type Item = &'g VertexDef;
    fn next(&mut self) -> Option<&'g VertexDef> {
        if !self.more() {
            return None;
        }
        let current = self.current();
        DefsVertexOfEdge::next(self);
        Some(current)
    }
}

/// `BRepGraph_DefsShellOfSolid`.
pub type DefsShellOfSolid<'g> = DefsOfParent<'g, ShellOfSolidTraits>;
/// `BRepGraph_DefsFaceOfShell`.
pub type DefsFaceOfShell<'g> = DefsOfParent<'g, FaceOfShellTraits>;
/// `BRepGraph_DefsEdgeOfWire`.
pub type DefsEdgeOfWire<'g> = DefsOfParent<'g, EdgeOfWireTraits>;
/// `BRepGraph_DefsWireOfFace`.
pub type DefsWireOfFace<'g> = DefsOfParent<'g, WireOfFaceTraits>;
/// `BRepGraph_DefsCoEdgeOfWire`.
pub type DefsCoEdgeOfWire<'g> = DefsOfParent<'g, CoEdgeOfWireTraits>;
/// `BRepGraph_DefsSolidOfCompSolid`.
pub type DefsSolidOfCompSolid<'g> = DefsOfParent<'g, SolidOfCompSolidTraits>;
/// `BRepGraph_DefsChildOfCompound`.
pub type DefsChildOfCompound<'g> = DefsOfParent<'g, ChildOfCompoundTraits>;
/// `BRepGraph_DefsOccurrenceOfProduct`.
pub type DefsOccurrenceOfProduct<'g> = DefsOfParent<'g, OccurrenceOfProductTraits>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::{FaceId, WireId};
    use crate::brep_graph_inc::ParityOrientation;

    // A face with three wires: the second wire removed, the third reference removed.
    fn face_with_wires() -> (BRepGraph, FaceId) {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let face = storage.append_face();
        let wires: Vec<WireId> = (0..3).map(|_| storage.append_wire()).collect();
        let refs: Vec<_> = wires
            .iter()
            .map(|wire| storage.attach_wire_to_face(face, *wire, ParityOrientation::FORWARD))
            .collect();
        storage.set_removed(wires[1], true);
        storage.set_removed(refs[2], true);
        (graph, face)
    }

    #[test]
    fn defs_skip_removed_children_and_refs() {
        let (graph, face) = face_with_wires();
        let mut iterator = DefsWireOfFace::new(&graph, face);
        assert!(iterator.more());
        assert_eq!(
            (iterator.current_id(), iterator.index()),
            (WireId::new(0), 0)
        );
        iterator.next();
        assert!(!iterator.more());
        // Past the end the release build still returns the last child found.
        assert_eq!(iterator.current_id(), WireId::new(0));
        assert!(!iterator.current_ref_id().is_valid());
        assert_eq!(DefsWireOfFace::new(&graph, face).count(), 1);
        assert!(!DefsWireOfFace::new(&graph, FaceId::new(4)).more());
    }

    #[test]
    fn vertex_of_edge_needs_an_active_edge() {
        let mut graph = BRepGraph::new();
        let edge = graph.inc_storage_mut().append_edge();
        let iterator = DefsVertexOfEdge::new(&graph, edge);
        assert!(!iterator.more());
        assert!(!iterator.current_ref_id().is_valid());
    }
}
