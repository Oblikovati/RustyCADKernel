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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_RefsIterator.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_RefsIterator`: the dense walk over the entries of one reference kind
//! (`RefIterator`) and the active child reference ids of one parent (`RefsOfParent`,
//! `RefsVertexOfEdge`).
use crate::brep_graph::parent_cursor::{EdgeVertexCursor, ParentCursor};
use crate::brep_graph::parent_traits::{
    ChildOfCompoundTraits, CoEdgeOfWireTraits, FaceOfShellTraits, OccurrenceOfProductTraits,
    ParentTraits, ShellOfSolidTraits, SolidOfCompSolidTraits, WireOfFaceTraits,
};
use crate::brep_graph::{
    BRepGraph, ChildRefTag, EdgeId, FaceRefTag, OccurrenceRefTag, ShellRefTag, SolidRefTag,
    TypedId, VertexRefId, VertexRefTag, WireRefTag,
};
use crate::brep_graph_inc::RefTag;

/// `BRepGraph_RefsIterator::Detail::RefTraits<Ref>`: the count and the entry accessor
/// of one reference kind, for every reference kind tag.
///
/// ```
/// use rustycad_tkbrep::brep_graph::refs_iterator::RefTraits;
/// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceRefTag};
/// assert_eq!(FaceRefTag::count(&BRepGraph::new()), 0);
/// ```
pub trait RefTraits: RefTag {
    /// `Count(theGraph)`: `Refs().<Kind>().Nb()`, removed entries included.
    fn count(graph: &BRepGraph) -> u32 {
        Self::store(graph.inc_storage()).items.len() as u32
    }

    /// `Get(theGraph, theRefId)`: `Refs().<Kind>().Entry(theRefId)`; an id out of range
    /// panics (undefined natively).
    fn get(graph: &BRepGraph, id: TypedId<Self>) -> &Self::Entry {
        &Self::store(graph.inc_storage()).items[id.index as usize]
    }
}

impl<T: RefTag> RefTraits for T {}

/// `BRepGraph_RefsIterator::RefIterator<RefType, TheFullTraverse>`: the entries of one
/// reference kind in id order, skipping removed ones unless `FULL` is true.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::refs_iterator::FaceRefIterator;
/// assert!(!FaceRefIterator::new(&BRepGraph::new()).more());
/// ```
pub struct RefIterator<'g, T: RefTag, const FULL: bool = false> {
    graph: &'g BRepGraph,
    current: TypedId<T>,
    length: TypedId<T>,
}

impl<'g, T: RefTag, const FULL: bool> RefIterator<'g, T, FULL> {
    /// `RefIterator(theGraph)`.
    pub fn new(graph: &'g BRepGraph) -> Self {
        Self::starting_at(graph, TypedId::start())
    }

    /// `RefIterator(theGraph, theStartId)`: the first (active) entry at or after `start`.
    pub fn starting_at(graph: &'g BRepGraph, start: TypedId<T>) -> Self {
        let mut iterator = Self {
            graph,
            current: start.max(TypedId::start()),
            length: TypedId::new(T::count(graph)),
        };
        iterator.skip_removed();
        iterator
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.current < self.length
    }

    /// `Next()`.
    pub fn next(&mut self) {
        self.current.pre_increment();
        self.skip_removed();
    }

    /// `Current()`: the current entry (panics past the end).
    pub fn current(&self) -> &'g T::Entry {
        T::get(self.graph, self.current)
    }

    /// `CurrentId()`.
    pub fn current_id(&self) -> TypedId<T> {
        self.current
    }

    fn skip_removed(&mut self) {
        if FULL {
            return;
        }
        while self.current < self.length && self.current.is_removed(self.graph) {
            self.current.pre_increment();
        }
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` entry.
impl<'g, T: RefTag, const FULL: bool> Iterator for RefIterator<'g, T, FULL> {
    type Item = &'g T::Entry;
    fn next(&mut self) -> Option<&'g T::Entry> {
        if !self.more() {
            return None;
        }
        let current = self.current();
        RefIterator::next(self);
        Some(current)
    }
}

/// `BRepGraph_RefsIterator::RefsOfParent<TraitsT>`: the active child reference ids of
/// a parent (the reference and its child both active).
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, WireId};
/// use rustycad_tkbrep::brep_graph::refs_iterator::CoEdgesOfWire;
/// assert!(!CoEdgesOfWire::new(&BRepGraph::new(), WireId::new(0)).more());
/// ```
pub struct RefsOfParent<'g, P: ParentTraits> {
    cursor: ParentCursor<'g, P>,
}

impl<'g, P: ParentTraits> RefsOfParent<'g, P> {
    /// `RefsOfParent(theGraph, theParent)`.
    pub fn new(graph: &'g BRepGraph, parent: TypedId<P::Parent>) -> Self {
        let mut cursor = ParentCursor::new(graph, parent);
        cursor.skip_removed();
        Self { cursor }
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.cursor.more()
    }

    /// `Next()`.
    pub fn next(&mut self) {
        self.cursor.index += 1;
        self.cursor.skip_removed();
    }

    /// `CurrentId()`: the current reference id (invalid past the end, where the native
    /// release build reads past the list).
    pub fn current_id(&self) -> P::Ref {
        self.cursor.ref_at_index().unwrap_or_default()
    }

    /// `Index()`: the position in the parent's relation list.
    pub fn index(&self) -> u32 {
        self.cursor.index
    }
}

/// The native `begin()`/`end()` range: yields each `CurrentId()`.
impl<P: ParentTraits> Iterator for RefsOfParent<'_, P> {
    type Item = P::Ref;
    fn next(&mut self) -> Option<P::Ref> {
        if !self.more() {
            return None;
        }
        let current = self.current_id();
        RefsOfParent::next(self);
        Some(current)
    }
}

/// `BRepGraph_RefsIterator::RefsVertexOfEdge`: the active start then end vertex
/// reference ids of an edge.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId};
/// use rustycad_tkbrep::brep_graph::refs_iterator::RefsVertexOfEdge;
/// assert!(!RefsVertexOfEdge::new(&BRepGraph::new(), EdgeId::new(0)).more());
/// ```
pub struct RefsVertexOfEdge<'g> {
    cursor: EdgeVertexCursor<'g>,
}

impl<'g> RefsVertexOfEdge<'g> {
    /// `RefsVertexOfEdge(theGraph, theEdgeId)`.
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

    /// `CurrentId()`: `refIdAt(myIndex)` (the end reference past the end, as natively;
    /// invalid without an edge).
    pub fn current_id(&self) -> VertexRefId {
        self.cursor.current_ref_id()
    }

    /// `Index()`.
    pub fn index(&self) -> u32 {
        self.cursor.index
    }
}

/// The native `begin()`/`end()` range: yields each `CurrentId()`.
impl Iterator for RefsVertexOfEdge<'_> {
    type Item = VertexRefId;
    fn next(&mut self) -> Option<VertexRefId> {
        if !self.more() {
            return None;
        }
        let current = self.current_id();
        RefsVertexOfEdge::next(self);
        Some(current)
    }
}

/// `BRepGraph_RefsShellOfSolid`.
pub type RefsShellOfSolid<'g> = RefsOfParent<'g, ShellOfSolidTraits>;
/// `BRepGraph_RefsFaceOfShell`.
pub type RefsFaceOfShell<'g> = RefsOfParent<'g, FaceOfShellTraits>;
/// `BRepGraph_RefsWireOfFace`.
pub type RefsWireOfFace<'g> = RefsOfParent<'g, WireOfFaceTraits>;
/// `BRepGraph_CoEdgesOfWire`.
pub type CoEdgesOfWire<'g> = RefsOfParent<'g, CoEdgeOfWireTraits>;
/// `BRepGraph_RefsSolidOfCompSolid`.
pub type RefsSolidOfCompSolid<'g> = RefsOfParent<'g, SolidOfCompSolidTraits>;
/// `BRepGraph_RefsChildOfCompound`.
pub type RefsChildOfCompound<'g> = RefsOfParent<'g, ChildOfCompoundTraits>;
/// `BRepGraph_RefsOccurrenceOfProduct`.
pub type RefsOccurrenceOfProduct<'g> = RefsOfParent<'g, OccurrenceOfProductTraits>;

/// `BRepGraph_ShellRefIterator`.
pub type ShellRefIterator<'g> = RefIterator<'g, ShellRefTag>;
/// `BRepGraph_FaceRefIterator`.
pub type FaceRefIterator<'g> = RefIterator<'g, FaceRefTag>;
/// `BRepGraph_WireRefIterator`.
pub type WireRefIterator<'g> = RefIterator<'g, WireRefTag>;
/// `BRepGraph_VertexRefIterator`.
pub type VertexRefIterator<'g> = RefIterator<'g, VertexRefTag>;
/// `BRepGraph_SolidRefIterator`.
pub type SolidRefIterator<'g> = RefIterator<'g, SolidRefTag>;
/// `BRepGraph_ChildRefIterator`.
pub type ChildRefIterator<'g> = RefIterator<'g, ChildRefTag>;
/// `BRepGraph_OccurrenceRefIterator`.
pub type OccurrenceRefIterator<'g> = RefIterator<'g, OccurrenceRefTag>;

/// `BRepGraph_FullShellRefIterator`: removed entries included.
pub type FullShellRefIterator<'g> = RefIterator<'g, ShellRefTag, true>;
/// `BRepGraph_FullFaceRefIterator`.
pub type FullFaceRefIterator<'g> = RefIterator<'g, FaceRefTag, true>;
/// `BRepGraph_FullWireRefIterator`.
pub type FullWireRefIterator<'g> = RefIterator<'g, WireRefTag, true>;
/// `BRepGraph_FullVertexRefIterator`.
pub type FullVertexRefIterator<'g> = RefIterator<'g, VertexRefTag, true>;
/// `BRepGraph_FullSolidRefIterator`.
pub type FullSolidRefIterator<'g> = RefIterator<'g, SolidRefTag, true>;
/// `BRepGraph_FullChildRefIterator`.
pub type FullChildRefIterator<'g> = RefIterator<'g, ChildRefTag, true>;
/// `BRepGraph_FullOccurrenceRefIterator`.
pub type FullOccurrenceRefIterator<'g> = RefIterator<'g, OccurrenceRefTag, true>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::{FaceRefId, ShellId};
    use crate::brep_graph_inc::ParityOrientation;

    #[test]
    fn refs_skip_removed_entries_and_children() {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let shell = storage.append_shell();
        let faces: Vec<_> = (0..3).map(|_| storage.append_face()).collect();
        let refs: Vec<FaceRefId> = faces
            .iter()
            .map(|face| storage.attach_face_to_shell(shell, *face, ParityOrientation::FORWARD))
            .collect();
        storage.set_removed(faces[0], true);
        storage.set_removed(refs[2], true);
        let collected: Vec<FaceRefId> = RefsFaceOfShell::new(&graph, shell).collect();
        assert_eq!(collected, vec![refs[1]]);
        assert_eq!(FaceRefIterator::new(&graph).count(), 2);
        assert_eq!(FullFaceRefIterator::new(&graph).count(), 3);
        let iterator = RefsFaceOfShell::new(&graph, ShellId::new(3));
        assert!(!iterator.more() && !iterator.current_id().is_valid());
    }
}
