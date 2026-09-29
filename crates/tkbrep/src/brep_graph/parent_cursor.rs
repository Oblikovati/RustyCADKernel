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
// Rust translation of the shared state and `skipRemoved` of
// BRepGraph_DefsIterator::DefsOfParent / BRepGraph_RefsIterator::RefsOfParent and of
// DefsVertexOfEdge / RefsVertexOfEdge (ModelingData/TKBRep/BRepGraph)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cursors the defs and refs iterators share: the native classes duplicate the same
//! members and `skipRemoved` loop; the port writes them once.
use crate::brep_graph::iterator::NodeTraits;
use crate::brep_graph::parent_traits::{ParentTraits, SlotId};
use crate::brep_graph::{BRepGraph, EdgeId, EdgeTag, TypedId, VertexRefId};
use crate::brep_graph_inc::EdgeDef;

/// The relation-list cursor of `DefsOfParent`/`RefsOfParent`.
pub(crate) struct ParentCursor<'g, P: ParentTraits> {
    pub(crate) graph: &'g BRepGraph,
    ref_ids: Option<&'g [P::Ref]>,
    pub(crate) index: u32,
    length: u32,
    nb_refs: u32,
    nb_children: u32,
}

impl<'g, P: ParentTraits> ParentCursor<'g, P> {
    /// The constructor body up to (excluding) `skipRemoved`: an invalid or removed
    /// parent leaves the cursor empty.
    pub(crate) fn new(graph: &'g BRepGraph, parent: TypedId<P::Parent>) -> Self {
        let mut cursor = Self {
            graph,
            ref_ids: None,
            index: 0,
            length: 0,
            nb_refs: 0,
            nb_children: 0,
        };
        if !P::is_parent_valid(graph, parent) {
            return cursor;
        }
        let ref_ids = P::ref_ids(graph, parent);
        cursor.ref_ids = Some(ref_ids);
        cursor.length = ref_ids.len() as u32;
        cursor.nb_refs = P::Ref::slot_count(graph);
        cursor.nb_children = P::Child::slot_count(graph);
        cursor
    }

    /// `More()`.
    pub(crate) fn more(&self) -> bool {
        self.ref_ids.is_some() && self.index < self.length
    }

    /// The reference at the cursor, `None` past the end.
    pub(crate) fn ref_at_index(&self) -> Option<P::Ref> {
        self.ref_ids?.get(self.index as usize).copied()
    }

    /// `skipRemoved`: moves to the first live reference with a live child at or after
    /// the cursor and returns that child.
    pub(crate) fn skip_removed(&mut self) -> Option<P::Child> {
        while let Some(reference) = self.ref_at_index() {
            if reference.is_live(self.graph, self.nb_refs) {
                let child = P::resolve_child(self.graph, reference);
                if child.is_live(self.graph, self.nb_children) {
                    return Some(child);
                }
            }
            self.index += 1;
        }
        None
    }
}

/// The start/end vertex-reference cursor of `DefsVertexOfEdge`/`RefsVertexOfEdge`.
pub(crate) struct EdgeVertexCursor<'g> {
    pub(crate) graph: &'g BRepGraph,
    edge: Option<&'g EdgeDef>,
    pub(crate) index: u32,
    length: u32,
    nb_vertex_refs: u32,
    nb_vertices: u32,
}

impl<'g> EdgeVertexCursor<'g> {
    /// The constructor: an invalid or removed edge leaves the cursor empty.
    pub(crate) fn new(graph: &'g BRepGraph, edge: EdgeId) -> Self {
        let mut cursor = Self {
            graph,
            edge: None,
            index: 0,
            length: 0,
            nb_vertex_refs: 0,
            nb_vertices: 0,
        };
        if !edge.is_valid_for(EdgeTag::count(graph)) || edge.is_removed(graph) {
            return cursor;
        }
        let storage = graph.inc_storage();
        cursor.edge = Some(storage.edge(edge));
        cursor.length = 2;
        cursor.nb_vertex_refs = storage.nb_vertex_refs();
        cursor.nb_vertices = storage.nb_vertices();
        cursor.skip_removed();
        cursor
    }

    /// `More()`.
    pub(crate) fn more(&self) -> bool {
        self.edge.is_some() && self.index < self.length
    }

    /// `refIdAt(myIndex)`: the start reference at index 0, the end reference otherwise
    /// (the invalid id without an edge).
    pub(crate) fn current_ref_id(&self) -> VertexRefId {
        match self.edge {
            Some(edge) if self.index == 0 => edge.start_vertex_ref_id,
            Some(edge) => edge.end_vertex_ref_id,
            None => VertexRefId::default(),
        }
    }

    /// `Next()`.
    pub(crate) fn next(&mut self) {
        self.index += 1;
        self.skip_removed();
    }

    // `skipRemoved`: a live reference (`Refs().Gen().IsRemoved` is false) whose vertex
    // is in range and active.
    fn skip_removed(&mut self) {
        while self.more() {
            let reference = self.current_ref_id();
            if reference.is_valid_for(self.nb_vertex_refs)
                && !self.graph.refs().generic().is_removed(reference.into())
            {
                let vertex = self
                    .graph
                    .inc_storage()
                    .vertex_ref(reference)
                    .child_vertex_id;
                if vertex.is_valid_for(self.nb_vertices) && !vertex.is_removed(self.graph) {
                    return;
                }
            }
            self.index += 1;
        }
    }
}
