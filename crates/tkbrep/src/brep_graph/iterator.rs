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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Iterator.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Iterator`: the dense walk over the definitions of one node kind, and
//! `BRepGraph_RootProductIterator`.
//!
//! `BRepGraph_Iterator<FaceDef>` is [`DefIterator<'_, FaceTag>`](DefIterator): the kind
//! tag names the definition type, so the eleven `NodeTraits` specialisations are one
//! blanket trait ([`NodeTraits`]). `TheFullTraverse` is the const parameter `FULL`.
use crate::brep_graph::{
    BRepGraph, CoEdgeTag, CompSolidTag, CompoundTag, EdgeTag, FaceTag, OccurrenceTag, ProductId,
    ProductTag, ShellTag, SolidTag, TypedId, VertexTag, WireTag,
};
use crate::brep_graph_inc::NodeTag;

/// `BRepGraph_IteratorDetail::NodeTraits<Def>` (and `BRepGraph_ReverseIterator::DefTraits`):
/// the count and the definition accessor of one node kind, for every node kind tag.
///
/// ```
/// use rustycad_tkbrep::brep_graph::iterator::NodeTraits;
/// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceTag};
/// let graph = BRepGraph::new();
/// assert_eq!(FaceTag::count(&graph), 0);
/// ```
pub trait NodeTraits: NodeTag {
    /// `Count(theGraph)`: `Topo().<Kind>().Nb()`, removed definitions included.
    fn count(graph: &BRepGraph) -> u32 {
        Self::store(graph.inc_storage()).items.len() as u32
    }

    /// `Get(theGraph, theId)`: `Topo().<Kind>().Definition(theId)`; an id out of range
    /// panics (undefined natively).
    fn get(graph: &BRepGraph, id: TypedId<Self>) -> &Self::Def {
        &Self::store(graph.inc_storage()).items[id.index as usize]
    }
}

impl<T: NodeTag> NodeTraits for T {}

/// `BRepGraph_Iterator<NodeType, TheFullTraverse>`: the definitions of one kind in id
/// order, skipping soft-removed ones unless `FULL` is true.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::iterator::FaceIterator;
/// let graph = BRepGraph::new();
/// let iterator = FaceIterator::new(&graph);
/// assert!(!iterator.more());
/// ```
pub struct DefIterator<'g, T: NodeTag, const FULL: bool = false> {
    graph: &'g BRepGraph,
    current: TypedId<T>,
    length: TypedId<T>,
}

impl<'g, T: NodeTag, const FULL: bool> DefIterator<'g, T, FULL> {
    /// `BRepGraph_Iterator(theGraph)`: positioned on the first (active) definition.
    pub fn new(graph: &'g BRepGraph) -> Self {
        Self::starting_at(graph, TypedId::start())
    }

    /// `BRepGraph_Iterator(theGraph, theStartId)`: positioned on the first (active)
    /// definition at or after `start` (a start below `Start()` is clamped).
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

    /// `Next()`: the next (active) definition.
    pub fn next(&mut self) {
        self.current.pre_increment();
        self.skip_removed();
    }

    /// `Current()`: the current definition (panics past the end).
    pub fn current(&self) -> &'g T::Def {
        T::get(self.graph, self.current)
    }

    /// `CurrentId()`: the current typed id.
    pub fn current_id(&self) -> TypedId<T> {
        self.current
    }

    // `skipRemoved`: a compile-time no-op for the full traversal.
    fn skip_removed(&mut self) {
        if FULL {
            return;
        }
        while self.current < self.length && self.current.is_removed(self.graph) {
            self.current.pre_increment();
        }
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` definition.
impl<'g, T: NodeTag, const FULL: bool> Iterator for DefIterator<'g, T, FULL> {
    type Item = &'g T::Def;
    fn next(&mut self) -> Option<&'g T::Def> {
        if !self.more() {
            return None;
        }
        let current = self.current();
        DefIterator::next(self);
        Some(current)
    }
}

/// `BRepGraph_SolidIterator`.
pub type SolidIterator<'g> = DefIterator<'g, SolidTag>;
/// `BRepGraph_ShellIterator`.
pub type ShellIterator<'g> = DefIterator<'g, ShellTag>;
/// `BRepGraph_FaceIterator`.
pub type FaceIterator<'g> = DefIterator<'g, FaceTag>;
/// `BRepGraph_WireIterator`.
pub type WireIterator<'g> = DefIterator<'g, WireTag>;
/// `BRepGraph_EdgeIterator`.
pub type EdgeIterator<'g> = DefIterator<'g, EdgeTag>;
/// `BRepGraph_VertexIterator`.
pub type VertexIterator<'g> = DefIterator<'g, VertexTag>;
/// `BRepGraph_CoEdgeIterator`.
pub type CoEdgeIterator<'g> = DefIterator<'g, CoEdgeTag>;
/// `BRepGraph_CompoundIterator`.
pub type CompoundIterator<'g> = DefIterator<'g, CompoundTag>;
/// `BRepGraph_CompSolidIterator`.
pub type CompSolidIterator<'g> = DefIterator<'g, CompSolidTag>;
/// `BRepGraph_ProductIterator`.
pub type ProductIterator<'g> = DefIterator<'g, ProductTag>;
/// `BRepGraph_OccurrenceIterator`.
pub type OccurrenceIterator<'g> = DefIterator<'g, OccurrenceTag>;

/// `BRepGraph_FullSolidIterator`: removed definitions included.
pub type FullSolidIterator<'g> = DefIterator<'g, SolidTag, true>;
/// `BRepGraph_FullShellIterator`.
pub type FullShellIterator<'g> = DefIterator<'g, ShellTag, true>;
/// `BRepGraph_FullFaceIterator`.
pub type FullFaceIterator<'g> = DefIterator<'g, FaceTag, true>;
/// `BRepGraph_FullWireIterator`.
pub type FullWireIterator<'g> = DefIterator<'g, WireTag, true>;
/// `BRepGraph_FullEdgeIterator`.
pub type FullEdgeIterator<'g> = DefIterator<'g, EdgeTag, true>;
/// `BRepGraph_FullVertexIterator`.
pub type FullVertexIterator<'g> = DefIterator<'g, VertexTag, true>;
/// `BRepGraph_FullCoEdgeIterator`.
pub type FullCoEdgeIterator<'g> = DefIterator<'g, CoEdgeTag, true>;
/// `BRepGraph_FullCompoundIterator`.
pub type FullCompoundIterator<'g> = DefIterator<'g, CompoundTag, true>;
/// `BRepGraph_FullCompSolidIterator`.
pub type FullCompSolidIterator<'g> = DefIterator<'g, CompSolidTag, true>;
/// `BRepGraph_FullProductIterator`.
pub type FullProductIterator<'g> = DefIterator<'g, ProductTag, true>;
/// `BRepGraph_FullOccurrenceIterator`.
pub type FullOccurrenceIterator<'g> = DefIterator<'g, OccurrenceTag, true>;

/// `BRepGraph_RootProductIterator`: the products no active occurrence references, in
/// the order of `BRepGraph::RootProductIds()`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::iterator::RootProductIterator;
/// let graph = BRepGraph::new();
/// assert_eq!(RootProductIterator::new(&graph).count(), 0);
/// ```
pub struct RootProductIterator<'g> {
    roots: &'g [ProductId],
    index: usize,
}

impl<'g> RootProductIterator<'g> {
    /// `BRepGraph_RootProductIterator(theGraph)`.
    pub fn new(graph: &'g BRepGraph) -> Self {
        Self {
            roots: graph.root_product_ids(),
            index: 0,
        }
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.index < self.roots.len()
    }

    /// `Next()`.
    pub fn next(&mut self) {
        self.index += 1;
    }

    /// `Current()`: the current root product (panics past the end).
    pub fn current(&self) -> ProductId {
        self.roots[self.index]
    }
}

/// The native `begin()`/`end()` range: yields each root product.
impl Iterator for RootProductIterator<'_> {
    type Item = ProductId;
    fn next(&mut self) -> Option<ProductId> {
        let current = self.roots.get(self.index).copied()?;
        self.index += 1;
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::FaceId;

    fn three_faces_second_removed() -> BRepGraph {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        for _ in 0..3 {
            storage.append_face();
        }
        storage.set_removed(FaceId::new(1), true);
        graph
    }

    #[test]
    fn removed_definitions_are_skipped_unless_full() {
        let graph = three_faces_second_removed();
        let mut ids = Vec::new();
        let mut iterator = FaceIterator::new(&graph);
        while iterator.more() {
            ids.push(iterator.current_id().index);
            iterator.next();
        }
        assert_eq!(ids, vec![0, 2]);
        assert_eq!(FullFaceIterator::new(&graph).count(), 3);
        assert_eq!(FaceIterator::new(&graph).count(), 2);
    }

    #[test]
    fn start_ids_clamp_and_skip() {
        let graph = three_faces_second_removed();
        let iterator = FaceIterator::starting_at(&graph, FaceId::new(1));
        assert_eq!(iterator.current_id(), FaceId::new(2));
        let full = FullFaceIterator::starting_at(&graph, FaceId::new(1));
        assert_eq!(full.current_id(), FaceId::new(1));
        assert!(!FaceIterator::starting_at(&graph, FaceId::new(7)).more());
        assert_eq!(FaceTag::count(&graph), 3);
        assert_eq!(RootProductIterator::new(&graph).count(), 0);
    }
}
