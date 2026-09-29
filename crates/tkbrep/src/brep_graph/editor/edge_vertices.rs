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
// Rust translation of EditorView::EdgeOps::RemoveVertex and ReplaceVertex of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx and SetStartVertexRefId and
// SetEndVertexRefId of BRepGraph_EditorView_Setters.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The members of `EditorView::EdgeOps` that edit the boundary vertex references.
use super::edges::EdgeOps;
use super::helpers::{has_any_active_usage, is_active_node, node_item, ref_item, require_unlocked};
use crate::brep_graph::mut_guard::MutGuard;
use crate::brep_graph::{BRepGraph, EdgeId, EdgeTag, NodeId, VertexId, VertexRefId};
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::standard::Failure;

/// Which boundary slot of an edge a setter writes.
#[derive(Clone, Copy)]
enum Slot {
    Start,
    End,
}

impl Slot {
    fn get(self, storage: &Storage, edge: EdgeId) -> VertexRefId {
        let def = storage.edge(edge);
        match self {
            Slot::Start => def.start_vertex_ref_id,
            Slot::End => def.end_vertex_ref_id,
        }
    }

    fn set(self, storage: &mut Storage, edge: EdgeId, reference: VertexRefId) {
        let def = storage.change_edge(edge);
        match self {
            Slot::Start => def.start_vertex_ref_id = reference,
            Slot::End => def.end_vertex_ref_id = reference,
        }
    }
}

impl EdgeOps<'_> {
    /// `RemoveVertex(theChildEdgeId, theVertexRefId)`: detaches one boundary vertex
    /// reference of an active edge (removing the reference and, when the vertex has no
    /// other use, the vertex); false when the reference is not an active boundary
    /// reference of the edge to an active vertex. Ids out of range return false (the
    /// release behaviour of the native assertions).
    pub fn remove_vertex(&mut self, edge: EdgeId, reference: VertexRefId) -> Result<bool, Failure> {
        let storage = self.graph.inc_storage();
        if !edge.is_valid_for(storage.nb_edges())
            || !reference.is_valid_for(storage.nb_vertex_refs())
            || !is_active_node(self.graph, edge)
        {
            return Ok(false);
        }
        let vertex = storage.vertex_ref(reference).child_vertex_id;
        let def = storage.edge(edge);
        if storage.is_removed(reference)
            || (def.start_vertex_ref_id != reference && def.end_vertex_ref_id != reference)
            || !vertex.is_valid()
            || !is_active_node(self.graph, vertex)
        {
            return Ok(false);
        }
        let slot = if def.start_vertex_ref_id == reference {
            Slot::Start
        } else {
            Slot::End
        };
        slot.set(self.graph.inc_storage_mut(), edge, VertexRefId::default());
        if !is_vertex_ref_owned_by_active_edge(self.graph.inc_storage(), reference)
            && !self.graph.editor().generic().remove_ref(reference.into())?
        {
            return Ok(false);
        }
        if !has_any_active_usage(self.graph, NodeId::from(vertex))? {
            self.graph
                .editor()
                .generic()
                .remove_subgraph(vertex.into())?;
        }
        self.graph.inc_storage_mut().rebind_vertex_edge(
            vertex,
            VertexId::default(),
            edge,
            reference,
        );
        self.graph.mark_modified(edge.into());
        Ok(true)
    }

    /// `ReplaceVertex(theChildEdgeId, theOldVertexRefId, theNewChildVertexId)`: a new
    /// boundary reference to the new vertex with the old orientation in the old
    /// reference's slot (the old reference is removed); the old reference itself when
    /// the vertex is unchanged; the invalid id for inactive inputs.
    pub fn replace_vertex(
        &mut self,
        edge: EdgeId,
        old_reference: VertexRefId,
        new_vertex: VertexId,
    ) -> Result<VertexRefId, Failure> {
        let storage = self.graph.inc_storage();
        if !edge.is_valid_for(storage.nb_edges())
            || !old_reference.is_valid_for(storage.nb_vertex_refs())
            || !new_vertex.is_valid_for(storage.nb_vertices())
            || !is_active_node(self.graph, edge)
            || !is_active_node(self.graph, new_vertex)
        {
            return Ok(VertexRefId::default());
        }
        let old_entry = storage.vertex_ref(old_reference).clone();
        let def = storage.edge(edge);
        if storage.is_removed(old_reference)
            || (def.start_vertex_ref_id != old_reference && def.end_vertex_ref_id != old_reference)
            || !old_entry.child_vertex_id.is_valid()
        {
            return Ok(VertexRefId::default());
        }
        if old_entry.child_vertex_id == new_vertex {
            return Ok(old_reference);
        }
        let slot = if def.start_vertex_ref_id == old_reference {
            Slot::Start
        } else {
            Slot::End
        };
        let new_reference = self.append_replacement_ref(edge, new_vertex, old_entry.orientation);
        slot.set(self.graph.inc_storage_mut(), edge, new_reference);
        if !is_vertex_ref_owned_by_active_edge(self.graph.inc_storage(), old_reference) {
            self.graph
                .editor()
                .generic()
                .remove_ref(old_reference.into())?;
        }
        let storage = self.graph.inc_storage_mut();
        storage.rebind_vertex_edge(old_entry.child_vertex_id, new_vertex, edge, old_reference);
        self.graph.mark_ref_modified(new_reference.into());
        self.graph.mark_modified(edge.into());
        Ok(new_reference)
    }

    // The replacement reference of `ReplaceVertex`, with its UID.
    fn append_replacement_ref(
        &mut self,
        edge: EdgeId,
        vertex: VertexId,
        orientation: crate::brep_graph_inc::ParityOrientation,
    ) -> VertexRefId {
        let storage = self.graph.inc_storage_mut();
        let reference = storage.append_vertex_ref();
        let entry = storage.change_vertex_ref(reference);
        entry.child_vertex_id = vertex;
        entry.parent_edge_id = edge;
        entry.orientation = orientation;
        self.graph.allocate_ref_uid(reference.into());
        reference
    }

    /// `SetStartVertexRefId(theEdge, theVertexRef)`: binds an active, unowned vertex
    /// reference (or the invalid id) as the start reference, rebinds the
    /// vertex-to-edge relation and removes the old reference. A locked edge or old
    /// reference raises Standard_ProgramError; an owned or removed new reference is
    /// ignored (the release behaviour of the native assertion).
    pub fn set_start_vertex_ref_id(
        &mut self,
        edge: EdgeId,
        reference: VertexRefId,
    ) -> Result<(), Failure> {
        self.set_boundary_ref(edge, reference, Slot::Start)
    }

    /// `SetStartVertexRefId(theMut, theVertexRef)`.
    pub fn set_start_vertex_ref_id_guarded(
        guard: &mut MutGuard<'_, EdgeTag>,
        reference: VertexRefId,
    ) -> Result<(), Failure> {
        set_guarded_boundary_ref(guard, reference, Slot::Start)
    }

    /// `SetEndVertexRefId(theEdge, theVertexRef)`.
    pub fn set_end_vertex_ref_id(
        &mut self,
        edge: EdgeId,
        reference: VertexRefId,
    ) -> Result<(), Failure> {
        self.set_boundary_ref(edge, reference, Slot::End)
    }

    /// `SetEndVertexRefId(theMut, theVertexRef)`.
    pub fn set_end_vertex_ref_id_guarded(
        guard: &mut MutGuard<'_, EdgeTag>,
        reference: VertexRefId,
    ) -> Result<(), Failure> {
        set_guarded_boundary_ref(guard, reference, Slot::End)
    }

    fn set_boundary_ref(
        &mut self,
        edge: EdgeId,
        reference: VertexRefId,
        slot: Slot,
    ) -> Result<(), Failure> {
        require_unlocked(
            self.graph,
            node_item(edge),
            "BRepGraph::EditorView: locked item",
        )?;
        if slot.get(self.graph.inc_storage(), edge) == reference {
            return Ok(());
        }
        let Some(rebind) = prepare_boundary_ref(self.graph, edge, reference, slot)? else {
            return Ok(());
        };
        apply_boundary_ref(self.graph, edge, reference, slot, rebind)?;
        self.graph.mark_modified(edge.into());
        Ok(())
    }
}

fn set_guarded_boundary_ref(
    guard: &mut MutGuard<'_, EdgeTag>,
    reference: VertexRefId,
    slot: Slot,
) -> Result<(), Failure> {
    let edge = guard.id();
    if slot.get(guard.graph().inc_storage(), edge) == reference {
        return Ok(());
    }
    let Some(rebind) = prepare_boundary_ref(guard.graph(), edge, reference, slot)? else {
        return Ok(());
    };
    // The slot is written through `Internal()`, which flags the guard.
    guard.mark_dirty();
    apply_boundary_ref(guard.graph_mut(), edge, reference, slot, rebind)
}

/// The old reference and the old and new vertices of a boundary rebind.
struct BoundaryRebind {
    old_reference: VertexRefId,
    old_vertex: VertexId,
    new_vertex: VertexId,
}

// The checks of both setters: an owned or removed new reference is ignored, then a
// locked old reference raises.
fn prepare_boundary_ref(
    graph: &BRepGraph,
    edge: EdgeId,
    reference: VertexRefId,
    slot: Slot,
) -> Result<Option<BoundaryRebind>, Failure> {
    let storage = graph.inc_storage();
    if !is_usable_unowned_vertex_ref(storage, reference) {
        return Ok(None);
    }
    let old_reference = slot.get(storage, edge);
    require_unlocked(
        graph,
        ref_item(old_reference),
        "BRepGraph::EditorView: locked item",
    )?;
    Ok(Some(BoundaryRebind {
        old_reference,
        old_vertex: vertex_from_ref(storage, old_reference),
        new_vertex: vertex_from_ref(storage, reference),
    }))
}

// Writes the slot, rebinds the vertex-to-edge relation and removes the old reference.
fn apply_boundary_ref(
    graph: &mut BRepGraph,
    edge: EdgeId,
    reference: VertexRefId,
    slot: Slot,
    rebind: BoundaryRebind,
) -> Result<(), Failure> {
    let storage = graph.inc_storage_mut();
    slot.set(storage, edge, reference);
    storage.rebind_vertex_edge(
        rebind.old_vertex,
        rebind.new_vertex,
        edge,
        VertexRefId::default(),
    );
    graph
        .editor()
        .generic()
        .remove_ref(rebind.old_reference.into())?;
    Ok(())
}

/// `vertexFromRef(theStorage, theRef)`: the child vertex of a reference in range.
fn vertex_from_ref(storage: &Storage, reference: VertexRefId) -> VertexId {
    if !reference.is_valid_for(storage.nb_vertex_refs()) {
        return VertexId::default();
    }
    storage.vertex_ref(reference).child_vertex_id
}

/// `isVertexRefOwnedByActiveEdge` / `isRefOwnedByAnyParent(VertexRefId)`: an active
/// edge of the reference's vertex uses it as a boundary reference.
pub(crate) fn is_vertex_ref_owned_by_active_edge(
    storage: &Storage,
    reference: VertexRefId,
) -> bool {
    if !reference.is_valid_for(storage.nb_vertex_refs()) || storage.is_removed(reference) {
        return false;
    }
    let vertex = storage.vertex_ref(reference).child_vertex_id;
    if !vertex.is_valid_for(storage.nb_vertices()) {
        return false;
    }
    storage
        .vertex_relations(vertex)
        .edge_ids
        .iter()
        .any(|edge| {
            if !edge.is_valid_for(storage.nb_edges()) || storage.is_removed(*edge) {
                return false;
            }
            let def = storage.edge(*edge);
            def.start_vertex_ref_id == reference || def.end_vertex_ref_id == reference
        })
}

/// `isUsableUnownedVertexRef`: the invalid id, or an active reference no active edge
/// uses as a boundary reference.
fn is_usable_unowned_vertex_ref(storage: &Storage, reference: VertexRefId) -> bool {
    !reference.is_valid()
        || (reference.is_valid_for(storage.nb_vertex_refs())
            && !storage.is_removed(reference)
            && !is_vertex_ref_owned_by_active_edge(storage, reference))
}
