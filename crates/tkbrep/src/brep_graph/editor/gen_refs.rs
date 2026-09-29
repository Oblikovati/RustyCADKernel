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
// Rust translation of the two EditorView::GenOps::RemoveRef overloads of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GenOps::RemoveRef`: the detachment of one reference.
use super::edge_vertices::is_vertex_ref_owned_by_active_edge;
use super::gen_ops::GenOps;
use super::helpers::{
    has_any_active_usage, is_active_node, is_expected_parent_kind_for_ref, is_ref_in_range,
    ref_item, require_mutable,
};
use super::removals::{detach_ordered_parent_ref, detach_ref_relations, is_listed_by_parent};
use crate::brep_graph::*;
use rustycad_tkernel::standard::Failure;

impl GenOps<'_> {
    /// `RemoveRef(theRef)`: detaches an active reference from its parent's list and the
    /// relation tables, marks it removed and modified, forgets its UID and tells the
    /// layers; true when it turned from active to removed. An invalid or removed
    /// reference gives false; a locked or guarded one raises Standard_ProgramError.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::{BRepGraph, RefId};
    /// let mut graph = BRepGraph::new();
    /// assert!(!graph.editor().generic().remove_ref(RefId::default()).unwrap());
    /// ```
    pub fn remove_ref(&mut self, reference: RefId) -> Result<bool, Failure> {
        if !reference.is_valid() {
            return Ok(false);
        }
        require_mutable(
            self.graph,
            ref_item(reference),
            "BRepGraph::EditorView::RemoveRef(): locked reference",
            "BRepGraph::EditorView::RemoveRef(): guard active on reference",
        )?;
        if reference.is_removed(self.graph) {
            return Ok(false);
        }
        // Natively `detachRefRelations` reads the entry unchecked (undefined behaviour
        // past the store); the port refuses the id before any write.
        self.graph.refs().generic().check_in_range(reference)?;
        let storage = self.graph.inc_storage_mut();
        detach_ref_relations(storage, reference);
        if !storage.mark_removed_ref(reference) {
            return Ok(false);
        }
        self.graph.mark_ref_modified(reference);
        let storage = self.graph.inc_storage_mut();
        let counter = storage.base_ref(reference).map_or(0, |base| base.uid);
        let uid = RefUid::new(reference.kind, counter);
        if uid.is_valid() {
            storage.unbind_ref_uid(&uid);
        }
        let layers = self.graph.layer_registry();
        layers.dispatch_on_ref_removed(Some(&*self.graph), reference);
        Ok(true)
    }

    /// `RemoveRef(theParent, theRef, theToPruneOrphanedChild)`: removes a reference
    /// only when the parent really owns it, pruning the child subgraph on request when
    /// no active use of it remains, and marks the parent modified; false otherwise
    /// (a parent kind that cannot own the reference kind is refused by a debug-only
    /// assertion natively and gives false here, as in the release build).
    pub fn remove_ref_of_parent(
        &mut self,
        parent: NodeId,
        reference: RefId,
        prune_orphaned_child: bool,
    ) -> Result<bool, Failure> {
        if !reference.is_valid() {
            return Ok(false);
        }
        require_mutable(
            self.graph,
            ref_item(reference),
            "BRepGraph::EditorView::RemoveRef(): locked reference",
            "BRepGraph::EditorView::RemoveRef(): guard active on reference",
        )?;
        if !is_expected_parent_kind_for_ref(parent, reference.kind)
            || !is_active_node(self.graph, parent)
            || !is_ref_in_range(self.graph, reference)
            || reference.is_removed(self.graph)
        {
            return Ok(false);
        }
        let child = self.graph.refs().generic().child_node(reference);
        let removed = if reference.kind == RefKind::VERTEX {
            self.remove_edge_vertex_ref(parent, reference, child, prune_orphaned_child)?
        } else {
            let listed = is_listed_by_parent(self.graph.inc_storage(), parent, reference);
            detach_ordered_parent_ref(self.graph, reference, listed, child, prune_orphaned_child)?
        };
        if !removed {
            return Ok(false);
        }
        self.graph.mark_modified(parent);
        Ok(true)
    }

    // The vertex case: the edge's matching boundary slots are cleared, the reference is
    // removed unless another edge uses it, and the vertex pruned on request.
    fn remove_edge_vertex_ref(
        &mut self,
        parent: NodeId,
        reference: RefId,
        child: NodeId,
        prune_orphaned_child: bool,
    ) -> Result<bool, Failure> {
        let (edge, vertex_ref) = (
            EdgeId::from_node_id(parent),
            VertexRefId::from_ref_id(reference),
        );
        let def = self.graph.inc_storage_mut().change_edge(edge);
        if def.start_vertex_ref_id != vertex_ref && def.end_vertex_ref_id != vertex_ref {
            return Ok(false);
        }
        if def.start_vertex_ref_id == vertex_ref {
            def.start_vertex_ref_id = VertexRefId::default();
        }
        if def.end_vertex_ref_id == vertex_ref {
            def.end_vertex_ref_id = VertexRefId::default();
        }
        if !is_vertex_ref_owned_by_active_edge(self.graph.inc_storage(), vertex_ref)
            && !self.remove_ref(reference)?
        {
            return Ok(false);
        }
        if prune_orphaned_child && child.is_valid() && !has_any_active_usage(self.graph, child)? {
            self.remove_subgraph(child)?;
        }
        Ok(true)
    }
}
