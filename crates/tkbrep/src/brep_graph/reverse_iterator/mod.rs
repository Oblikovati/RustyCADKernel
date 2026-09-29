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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_ReverseIterator.hxx
// (DefTraits, ParentsOf and the type aliases) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_ReverseIterator`: the parents of one node, one level up, through the
//! reverse relation lists, skipping removed parents.
//!
//! [`ParentsOf`] walks a list of parent ids, [`EdgeParentsOf`] derives the wires or
//! faces of an edge from its coedges (first occurrence only), [`IdsOfRefs`] reads the
//! parent of each reference of a list, and [`LookupParentRefsOf`] finds the reference
//! each parent holds to the child. `DefTraits<TypedId>` is the node kind's
//! [`NodeTraits`](crate::brep_graph::iterator::NodeTraits).
mod edge_parents;
mod ids_of_refs;
mod lookup;

pub use crate::brep_graph::iterator::NodeTraits as DefTraits;
pub use edge_parents::{
    EdgeParentTraits, EdgeParentsOf, FaceFromEdgeCoEdgeTraits, FacesOfEdge,
    WireFromEdgeCoEdgeTraits, WiresOfEdge,
};
pub use ids_of_refs::{
    CompSolidFromSolidRefTraits, CompoundFromChildRefTraits, FaceFromWireRefTraits, IdsOfRefs,
    IdsOfRefsTraits, OccurrenceFromOccurrenceRefTraits, ProductFromOccurrenceRefTraits,
    ShellFromFaceRefTraits, SolidFromShellRefTraits,
};
pub use lookup::{
    EdgeOfVertexRefTraits, LookupParentRefsOf, LookupTraits, ParentRef, WireOfCoEdgeUsageTraits,
};

use crate::brep_graph::{BRepGraph, CoEdgeTag, EdgeTag, TypedId};
use crate::brep_graph_inc::NodeTag;

/// `ParentsOf<TypedIdT>`: the active parents of a relation list of parent ids, with
/// indexed access to the raw list.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId};
/// use rustycad_tkbrep::brep_graph::reverse_iterator::EdgesOfVertex;
/// let graph = BRepGraph::new();
/// let edges: Vec<EdgeId> = Vec::new();
/// assert!(!EdgesOfVertex::new(&graph, &edges).more());
/// ```
pub struct ParentsOf<'g, T: NodeTag> {
    graph: &'g BRepGraph,
    parents: &'g [TypedId<T>],
    nb_parents: u32,
    index: u32,
}

impl<'g, T: NodeTag> ParentsOf<'g, T> {
    /// `ParentsOf(theGraph, theParents)`: positioned on the first active parent.
    pub fn new(graph: &'g BRepGraph, parents: &'g [TypedId<T>]) -> Self {
        Self::starting_at(graph, parents, 0)
    }

    /// `ParentsOf(theGraph, theParents, theStartIndex)`: the first active parent at or
    /// after `start_index` (for resumable iteration).
    pub fn starting_at(graph: &'g BRepGraph, parents: &'g [TypedId<T>], start_index: u32) -> Self {
        let mut iterator = Self {
            graph,
            parents,
            nb_parents: T::count(graph),
            index: start_index,
        };
        iterator.skip_removed();
        iterator
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        (self.index as usize) < self.parents.len()
    }

    /// `Next()`.
    pub fn next(&mut self) {
        self.index += 1;
        self.skip_removed();
    }

    /// `CurrentId()`: the current parent (panics past the end).
    pub fn current_id(&self) -> TypedId<T> {
        self.parents[self.index as usize]
    }

    /// `Current()`: the same as [`current_id`](Self::current_id).
    pub fn current(&self) -> TypedId<T> {
        self.current_id()
    }

    /// `Definition()`: the current parent's definition.
    pub fn definition(&self) -> &'g T::Def {
        T::get(self.graph, self.current_id())
    }

    /// `Index()`: the position in the list.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// `Size()`: the raw list size, removed parents included.
    pub fn size(&self) -> usize {
        self.parents.len()
    }

    /// `Value(theIndex)`: the raw parent at `index` (removal not checked; panics out of
    /// range).
    pub fn value(&self, index: usize) -> TypedId<T> {
        self.parents[index]
    }

    fn skip_removed(&mut self) {
        while let Some(parent) = self.parents.get(self.index as usize) {
            if parent.is_valid_for(self.nb_parents) && !parent.is_removed(self.graph) {
                return;
            }
            self.index += 1;
        }
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` id.
impl<T: NodeTag> Iterator for ParentsOf<'_, T> {
    type Item = TypedId<T>;
    fn next(&mut self) -> Option<TypedId<T>> {
        if !self.more() {
            return None;
        }
        let current = self.current_id();
        ParentsOf::next(self);
        Some(current)
    }
}

/// `BRepGraph_EdgesOfVertex`: vertex to parent edges.
pub type EdgesOfVertex<'g> = ParentsOf<'g, EdgeTag>;
/// `BRepGraph_CoEdgesOfEdge`: edge to its coedges.
pub type CoEdgesOfEdge<'g> = ParentsOf<'g, CoEdgeTag>;
/// `BRepGraph_CompoundsOfVertex`.
pub type CompoundsOfVertex<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_CompoundsOfEdge`.
pub type CompoundsOfEdge<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_CompoundsOfCoEdge`.
pub type CompoundsOfCoEdge<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_FacesOfWire`: wire to parent faces.
pub type FacesOfWire<'g> = IdsOfRefs<'g, FaceFromWireRefTraits>;
/// `BRepGraph_CompoundsOfWire`.
pub type CompoundsOfWire<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_ShellsOfFace`: face to parent shells.
pub type ShellsOfFace<'g> = IdsOfRefs<'g, ShellFromFaceRefTraits>;
/// `BRepGraph_CompoundsOfFace`.
pub type CompoundsOfFace<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_SolidsOfShell`: shell to parent solids.
pub type SolidsOfShell<'g> = IdsOfRefs<'g, SolidFromShellRefTraits>;
/// `BRepGraph_CompoundsOfShell`.
pub type CompoundsOfShell<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_CompSolidsOfSolid`: solid to parent compsolids.
pub type CompSolidsOfSolid<'g> = IdsOfRefs<'g, CompSolidFromSolidRefTraits>;
/// `BRepGraph_CompoundsOfSolid`.
pub type CompoundsOfSolid<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_CompoundsOfCompSolid`.
pub type CompoundsOfCompSolid<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_CompoundsOfCompound`.
pub type CompoundsOfCompound<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_CompoundsOfChild`: any child to parent compounds.
pub type CompoundsOfChild<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_OccurrencesOfProduct`: product to its occurrences.
pub type OccurrencesOfProduct<'g> = IdsOfRefs<'g, OccurrenceFromOccurrenceRefTraits>;
/// `BRepGraph_OccurrencesOfChild`.
pub type OccurrencesOfChild<'g> = IdsOfRefs<'g, OccurrenceFromOccurrenceRefTraits>;
/// `BRepGraph_ProductsOfOccurrence`: occurrence to parent products.
pub type ProductsOfOccurrence<'g> = IdsOfRefs<'g, ProductFromOccurrenceRefTraits>;
/// `BRepGraph_RefsFacesOfWire`: parent faces with the wire reference.
pub type RefsFacesOfWire<'g> = IdsOfRefs<'g, FaceFromWireRefTraits>;
/// `BRepGraph_RefsShellsOfFace`.
pub type RefsShellsOfFace<'g> = IdsOfRefs<'g, ShellFromFaceRefTraits>;
/// `BRepGraph_RefsSolidsOfShell`.
pub type RefsSolidsOfShell<'g> = IdsOfRefs<'g, SolidFromShellRefTraits>;
/// `BRepGraph_RefsWiresOfCoEdge`: parent wires with the coedge usage.
pub type RefsWiresOfCoEdge<'g> = LookupParentRefsOf<'g, WireOfCoEdgeUsageTraits>;
/// `BRepGraph_RefsEdgesOfVertex`: parent edges with the vertex reference.
pub type RefsEdgesOfVertex<'g> = LookupParentRefsOf<'g, EdgeOfVertexRefTraits>;
/// `BRepGraph_RefsCompSolidsOfSolid`.
pub type RefsCompSolidsOfSolid<'g> = IdsOfRefs<'g, CompSolidFromSolidRefTraits>;
/// `BRepGraph_RefsCompoundsOfChild`.
pub type RefsCompoundsOfChild<'g> = IdsOfRefs<'g, CompoundFromChildRefTraits>;
/// `BRepGraph_RefsProductsOfOccurrence`.
pub type RefsProductsOfOccurrence<'g> = IdsOfRefs<'g, ProductFromOccurrenceRefTraits>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::EdgeId;

    #[test]
    fn parents_skip_removed_and_out_of_range_ids() {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let edges: Vec<EdgeId> = (0..3).map(|_| storage.append_edge()).collect();
        storage.set_removed(edges[1], true);
        let list = vec![edges[1], EdgeId::new(9), edges[2], edges[0]];
        let collected: Vec<EdgeId> = EdgesOfVertex::new(&graph, &list).collect();
        assert_eq!(collected, vec![edges[2], edges[0]]);
        let resumed = EdgesOfVertex::starting_at(&graph, &list, 3);
        assert_eq!((resumed.current(), resumed.index()), (edges[0], 3));
        assert_eq!((resumed.size(), resumed.value(0)), (4, edges[1]));
    }
}
