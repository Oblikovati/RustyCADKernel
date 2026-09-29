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
// Rust support for the `for (BRepGraph_Iterator<Def> it(g); it.More(); it.Next())
// it.CurrentId()` loops of BRepGraph_Copy.cxx, BRepGraph_Compact.cxx and
// BRepGraph_Deduplicate.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ids a definition iterator visits, collected up front.
//!
//! The algorithms walk a kind with `BRepGraph_Iterator` (active only) or the `Full`
//! iterators while they write other kinds or other graphs; collecting the ids first
//! releases the graph borrow. They collect only where the walked kind's removed plane
//! cannot change during the loop, so the ids are the ones the native loop visits.
use crate::brep_graph::iterator::DefIterator;
use crate::brep_graph::{BRepGraph, NodeTag, TypedId};

/// The ids `BRepGraph_Iterator<Def, FULL>` visits in order: every slot for the full
/// traversal, the active slots otherwise.
pub(crate) fn def_ids<T: NodeTag, const FULL: bool>(graph: &BRepGraph) -> Vec<TypedId<T>> {
    let mut iterator = DefIterator::<T, FULL>::new(graph);
    let mut ids = Vec::new();
    while iterator.more() {
        ids.push(iterator.current_id());
        iterator.next();
    }
    ids
}

/// The active ids (`BRepGraph_Iterator<Def>`).
pub(crate) fn active_ids<T: NodeTag>(graph: &BRepGraph) -> Vec<TypedId<T>> {
    def_ids::<T, false>(graph)
}

/// Every id (`BRepGraph_Full*Iterator`).
pub(crate) fn full_ids<T: NodeTag>(graph: &BRepGraph) -> Vec<TypedId<T>> {
    def_ids::<T, true>(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::{VertexId, VertexTag};
    use rustycad_tkmath::gp::Point3;

    #[test]
    fn active_ids_skip_removed_slots_full_ids_do_not() {
        let mut graph = BRepGraph::new();
        for _ in 0..3 {
            graph.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1e-7);
        }
        graph.inc_storage_mut().set_removed(VertexId::new(1), true);
        let active = active_ids::<VertexTag>(&graph);
        assert_eq!(active, vec![VertexId::new(0), VertexId::new(2)]);
        assert_eq!(full_ids::<VertexTag>(&graph).len(), 3);
    }
}
