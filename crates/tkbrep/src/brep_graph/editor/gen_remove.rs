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
// Rust translation of EditorView::GenOps::RemoveNode (with its markOwnedRefRemoved
// lambda, which ReplaceNode and CleanupRemovedReferences repeat) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GenOps::RemoveNode`: the soft deletion of one node.
use super::gen_ops::GenOps;
use super::helpers::{
    is_active_node, is_node_in_range, is_ref_in_range, node_item, remove_from_root_products,
    require_mutable,
};
use crate::brep_graph::*;
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::standard::Failure;

impl GenOps<'_> {
    /// `RemoveNode(theNode)`: marks an active node removed after propagating the
    /// generation upwards (not for vertices, coedges, products and occurrences) and
    /// telling the layers while the parent context still exists; its owned
    /// references and representations are removed and its relations unbound. An
    /// invalid or inactive node (or one out of range, the release behaviour of the
    /// native assertion) is ignored; a locked or guarded node raises
    /// Standard_ProgramError.
    pub fn remove_node(&mut self, node: NodeId) -> Result<(), Failure> {
        if !node.is_valid() {
            return Ok(());
        }
        require_mutable(
            self.graph,
            node_item(node),
            "BRepGraph::EditorView::RemoveNode(): locked node",
            "BRepGraph::EditorView::RemoveNode(): guard active on node",
        )?;
        if !is_node_in_range(self.graph, node) || !is_active_node(self.graph, node) {
            return Ok(());
        }
        propagate_before_removal(self.graph, node);
        let layers = self.graph.layer_registry();
        layers.dispatch_on_node_removed(Some(&*self.graph), node);
        match node.kind {
            NodeKind::EDGE => remove_edge_uses(self.graph, EdgeId::from_node_id(node)),
            NodeKind::CO_EDGE => {
                remove_co_edge_uses(self.graph.inc_storage_mut(), CoEdgeId::from_node_id(node))
            }
            NodeKind::FACE => remove_face_uses(self.graph, FaceId::from_node_id(node)),
            _ => {
                for reference in owned_container_refs(self.graph.inc_storage(), node) {
                    mark_owned_ref_removed(self.graph, reference);
                }
            }
        }
        finish_node_removal(self.graph, node);
        Ok(())
    }
}

/// The upward generation propagation that precedes a removal (not for vertices,
/// coedges, products and occurrences).
pub(crate) fn propagate_before_removal(graph: &mut BRepGraph, node: NodeId) {
    let skipped = [
        NodeKind::VERTEX,
        NodeKind::CO_EDGE,
        NodeKind::PRODUCT,
        NodeKind::OCCURRENCE,
    ];
    if !skipped.contains(&node.kind) {
        graph.propagate_subtree_gen(node);
    }
}

/// `markOwnedRefRemoved(theRef)` (a lambda natively): an active reference in range is
/// removed, marked modified, forgotten by the UID index and announced to the layers.
pub(crate) fn mark_owned_ref_removed(graph: &mut BRepGraph, reference: RefId) {
    if !is_ref_in_range(graph, reference)
        || reference.is_removed(graph)
        || !graph.inc_storage_mut().mark_removed_ref(reference)
    {
        return;
    }
    graph.mark_ref_modified(reference);
    let storage = graph.inc_storage_mut();
    let counter = storage.base_ref(reference).map_or(0, |base| base.uid);
    let uid = RefUid::new(reference.kind, counter);
    if uid.is_valid() {
        storage.unbind_ref_uid(&uid);
    }
    graph
        .layer_registry()
        .dispatch_on_ref_removed(Some(&*graph), reference);
}

// The edge case: its boundary references removed, the vertex relations unbound, the
// curve and polygon records removed.
fn remove_edge_uses(graph: &mut BRepGraph, edge: EdgeId) {
    let storage = graph.inc_storage();
    let def = storage.edge(edge).clone();
    let child = |reference: VertexRefId| {
        if reference.is_valid_for(storage.nb_vertex_refs()) {
            storage.vertex_ref(reference).child_vertex_id
        } else {
            VertexId::default()
        }
    };
    let (start, end) = (child(def.start_vertex_ref_id), child(def.end_vertex_ref_id));
    mark_owned_ref_removed(graph, def.start_vertex_ref_id.into());
    mark_owned_ref_removed(graph, def.end_vertex_ref_id.into());
    let storage = graph.inc_storage_mut();
    storage.rebind_vertex_edge(start, VertexId::default(), edge, VertexRefId::default());
    storage.rebind_vertex_edge(end, VertexId::default(), edge, VertexRefId::default());
    let def = storage.edge(edge).clone();
    if def.curve3d_rep_id.is_valid() {
        storage.mark_rep_removed(def.curve3d_rep_id.into());
    }
    if def.polygon3d_rep_id.is_valid() {
        storage.mark_rep_removed(def.polygon3d_rep_id.into());
    }
}

// The coedge case: its three representation records removed and its wire use
// detached.
fn remove_co_edge_uses(storage: &mut Storage, co_edge: CoEdgeId) {
    super::helpers::mark_co_edge_representations_removed(storage, co_edge);
    let wire = storage.co_edge(co_edge).parent_wire_id;
    if wire.is_valid_for(storage.nb_wires()) {
        storage.detach_co_edge_use(wire, co_edge);
    }
}

// The face case: the face is removed first (so the reference cleanup does not mark it
// modified again), then its records, its wire references, its parent references, and
// the face binding of every coedge (removed ones included) on it.
fn remove_face_uses(graph: &mut BRepGraph, face: FaceId) {
    let storage = graph.inc_storage_mut();
    storage.mark_removed(face.into());
    let def = storage.face(face).clone();
    if def.surface_rep_id.is_valid() {
        storage.mark_rep_removed(def.surface_rep_id.into());
    }
    if def.triangulation_rep_id.is_valid() {
        storage.mark_rep_removed(def.triangulation_rep_id.into());
    }
    while let Some(reference) = graph
        .inc_storage()
        .face_relations(face)
        .wire_ref_ids
        .first()
        .copied()
    {
        mark_owned_ref_removed(graph, reference.into());
        graph
            .inc_storage_mut()
            .detach_wire_from_face(face, reference);
    }
    while let Some(reference) = graph
        .inc_storage()
        .face_relations(face)
        .parent_face_ref_ids
        .first()
        .copied()
    {
        let shell = graph.inc_storage().face_ref(reference).parent_shell_id;
        mark_owned_ref_removed(graph, reference.into());
        graph
            .inc_storage_mut()
            .detach_face_from_shell(shell, reference);
    }
    unbind_face_co_edges(graph.inc_storage_mut(), face);
}

// Every coedge on the face (the full iterator: RemoveSubgraph removed them already)
// loses its face and its face-scoped records.
fn unbind_face_co_edges(storage: &mut Storage, face: FaceId) {
    for index in 0..storage.nb_co_edges() {
        let co_edge = CoEdgeId::new(index);
        if storage.co_edge(co_edge).face_id != face {
            continue;
        }
        storage.change_co_edge(co_edge).face_id = FaceId::default();
        let def = storage.co_edge(co_edge).clone();
        let reps: [RepId; 3] = [
            def.curve2d_rep_id.into(),
            def.polygon2d_rep_id.into(),
            def.polygon_on_tri_rep_id.into(),
        ];
        for rep in reps.into_iter().filter(|rep| rep.is_valid()) {
            storage.mark_rep_removed(rep);
        }
    }
}

// The owned references of a shell, solid, compsolid, compound or product (none for
// the other kinds), in list order.
fn owned_container_refs(storage: &Storage, node: NodeId) -> Vec<RefId> {
    fn refs<I: Copy + Into<RefId>>(ids: &[I]) -> Vec<RefId> {
        ids.iter().map(|id| (*id).into()).collect()
    }
    match node.kind {
        NodeKind::SHELL => refs(
            &storage
                .shell_relations(ShellId::from_node_id(node))
                .face_ref_ids,
        ),
        NodeKind::SOLID => refs(
            &storage
                .solid_relations(SolidId::from_node_id(node))
                .shell_ref_ids,
        ),
        NodeKind::COMP_SOLID => refs(
            &storage
                .comp_solid_relations(CompSolidId::from_node_id(node))
                .solid_ref_ids,
        ),
        NodeKind::COMPOUND => refs(
            &storage
                .compound_relations(CompoundId::from_node_id(node))
                .child_ref_ids,
        ),
        NodeKind::PRODUCT => refs(
            &storage
                .product_relations(ProductId::from_node_id(node))
                .occurrence_ref_ids,
        ),
        _ => Vec::new(),
    }
}

/// The shared tail of `RemoveNode` and `ReplaceNode`: the node is marked removed (and
/// leaves the document roots), its generations bumped, its UID forgotten and its
/// cached shape dropped.
pub(crate) fn finish_node_removal(graph: &mut BRepGraph, node: NodeId) {
    let Some(uid_counter) = graph.topo_entity(node).map(|entity| entity.uid) else {
        graph.inc_storage().unbind_current_shape(node);
        return;
    };
    if !node.is_removed(graph) {
        graph.inc_storage_mut().mark_removed(node);
        if node.kind == NodeKind::PRODUCT {
            remove_from_root_products(graph, ProductId::from_node_id(node));
        }
    }
    if let Some(entity) = graph.change_topo_entity(node) {
        entity.own_gen = entity.own_gen.wrapping_add(1);
        entity.subtree_gen = entity.subtree_gen.wrapping_add(1);
    }
    let uid = Uid::new(node.kind, uid_counter);
    if uid.is_valid() {
        graph.inc_storage_mut().unbind_node_uid(&uid);
    }
    graph.inc_storage().unbind_current_shape(node);
}
