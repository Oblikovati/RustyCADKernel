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
// Rust translation of markModified, markRefModified, markParentSubtreeGen and
// propagateSubtreeGen of ModelingData/TKBRep/BRepGraph/BRepGraph.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The split-generation bookkeeping: a direct mutation bumps OwnGen and SubtreeGen on
//! the node and SubtreeGen on every ancestor (once per propagation wave).
use super::BRepGraph;
use crate::brep_graph::*;

impl BRepGraph {
    /// `markModified(theNodeId)` (and the overload that receives the entity, which is
    /// the same lookup here): bumps OwnGen and SubtreeGen, stamps the wave, then queues
    /// the node in deferred mode or dispatches the event and propagates upwards.
    pub(crate) fn mark_modified(&mut self, node: NodeId) {
        if !node.is_valid() {
            return;
        }
        let wave_needed = self.topo_entity(node).is_some();
        if !wave_needed {
            return;
        }
        let wave = self.inc_storage().advance_propagation_wave();
        if let Some(entity) = self.change_topo_entity(node) {
            entity.own_gen = entity.own_gen.wrapping_add(1);
            entity.subtree_gen = entity.subtree_gen.wrapping_add(1);
            entity.last_prop_wave = wave;
        }
        if self.inc_storage().deferred_mode() {
            self.inc_storage_mut().change_deferred_modified().push(node);
            return;
        }
        self.dispatch_node_modified(node);
        self.propagate_subtree_gen(node);
    }

    // The layer dispatch of `markModified`: the registry reads the graph in place, so
    // a layer callback sees the registry too.
    fn dispatch_node_modified(&mut self, node: NodeId) {
        let layers = self.layer_registry();
        if layers.has_modification_subscribers() {
            layers.dispatch_node_modified(Some(&*self), node);
        }
    }

    fn dispatch_ref_modified(&mut self, reference: RefId) {
        let layers = self.layer_registry();
        if layers.has_ref_modification_subscribers() {
            layers.dispatch_ref_modified(Some(&*self), reference);
        }
    }

    /// `markRefModified(theRefId)`: queues or dispatches an active reference, then marks
    /// the active parent definition modified (the vertex reference marks every active
    /// edge of its vertex that uses it).
    pub(crate) fn mark_ref_modified(&mut self, reference: RefId) {
        if !reference.is_valid() {
            return;
        }
        if !reference.is_removed(self) {
            if self.inc_storage().deferred_mode() {
                self.inc_storage_mut()
                    .change_deferred_ref_modified()
                    .push(reference);
            } else {
                self.dispatch_ref_modified(reference);
            }
        }
        for parent in self.ref_parents_to_mark(reference) {
            self.mark_modified(parent);
        }
    }

    // The definitions `markRefModified` marks, in native order.
    fn ref_parents_to_mark(&self, reference: RefId) -> Vec<NodeId> {
        let storage = self.inc_storage();
        if reference.kind == RefKind::VERTEX {
            return self.vertex_ref_edges(VertexRefId::from_ref_id(reference));
        }
        let parent = ref_parent_node(storage, reference);
        if parent.is_valid() && !parent.is_removed(self) {
            return vec![parent];
        }
        Vec::new()
    }

    // The active edges of the reference's vertex whose start or end is the reference.
    fn vertex_ref_edges(&self, reference: VertexRefId) -> Vec<NodeId> {
        let storage = self.inc_storage();
        if !reference.is_valid_for(storage.nb_vertex_refs()) {
            return Vec::new();
        }
        let vertex = storage.vertex_ref(reference).child_vertex_id;
        if !vertex.is_valid_for(storage.nb_vertices()) {
            return Vec::new();
        }
        let edges = &storage.vertex_relations(vertex).edge_ids;
        edges
            .iter()
            .filter(|edge| edge.is_valid_for(storage.nb_edges()))
            .filter(|edge| {
                let def = storage.edge(**edge);
                !edge.is_removed(self)
                    && (def.start_vertex_ref_id == reference || def.end_vertex_ref_id == reference)
            })
            .map(|edge| NodeId::from(*edge))
            .collect()
    }

    /// `markParentSubtreeGen(theParentId)`: bumps SubtreeGen only (the parent's own data
    /// did not change), once per wave, then continues upwards.
    pub(crate) fn mark_parent_subtree_gen(&mut self, parent: NodeId) {
        let wave = self.inc_storage().propagation_wave();
        let Some(entity) = self.change_topo_entity(parent) else {
            return;
        };
        if entity.last_prop_wave == wave {
            return;
        }
        entity.last_prop_wave = wave;
        entity.subtree_gen = entity.subtree_gen.wrapping_add(1);
        self.propagate_subtree_gen(parent);
    }

    /// `propagateSubtreeGen(theNodeId)`: marks the parents of the node through the
    /// relation tables (vertex, edge, coedge, wire, face, shell and occurrence parents;
    /// the parent occurrences of the other kinds).
    pub(crate) fn propagate_subtree_gen(&mut self, node: NodeId) {
        for parent in self.subtree_parents(node) {
            self.mark_parent_subtree_gen(parent);
        }
    }

    // The parents `propagateSubtreeGen` visits, in native order.
    fn subtree_parents(&self, node: NodeId) -> Vec<NodeId> {
        let storage = self.inc_storage();
        match node.kind {
            NodeKind::VERTEX => {
                let vertex = VertexId::from_node_id(node);
                storage
                    .vertex_relations(vertex)
                    .edge_ids
                    .iter()
                    .map(|e| NodeId::from(*e))
                    .collect()
            }
            NodeKind::EDGE => edge_parent_wires(storage, EdgeId::from_node_id(node)),
            NodeKind::CO_EDGE => co_edge_parent_wire(storage, CoEdgeId::from_node_id(node)),
            NodeKind::WIRE => active_ref_parents(
                storage,
                &storage
                    .wire_relations(WireId::from_node_id(node))
                    .parent_wire_ref_ids,
                |e: &crate::brep_graph_inc::WireRef| e.parent_face_id.into(),
            ),
            NodeKind::FACE => active_ref_parents(
                storage,
                &storage
                    .face_relations(FaceId::from_node_id(node))
                    .parent_face_ref_ids,
                |e: &crate::brep_graph_inc::FaceRef| e.parent_shell_id.into(),
            ),
            NodeKind::SHELL => active_ref_parents(
                storage,
                &storage
                    .shell_relations(ShellId::from_node_id(node))
                    .parent_shell_ref_ids,
                |e: &crate::brep_graph_inc::ShellRef| e.parent_solid_id.into(),
            ),
            NodeKind::OCCURRENCE => {
                occurrence_parent_products(storage, OccurrenceId::from_node_id(node))
            }
            _ => occurrences_of_node(storage, node),
        }
    }
}

/// The parent definition of a non-vertex reference in range (its parent face, shell,
/// solid, compsolid, compound or product), invalid otherwise; shared by
/// `markRefModified` and the cache generations.
pub(crate) fn ref_parent_node(
    storage: &crate::brep_graph_inc::Storage,
    reference: RefId,
) -> NodeId {
    match reference.kind {
        RefKind::WIRE => ref_parent(storage, WireRefId::from_ref_id(reference), |e| {
            e.parent_face_id
        }),
        RefKind::FACE => ref_parent(storage, FaceRefId::from_ref_id(reference), |e| {
            e.parent_shell_id
        }),
        RefKind::SHELL => ref_parent(storage, ShellRefId::from_ref_id(reference), |e| {
            e.parent_solid_id
        }),
        RefKind::SOLID => ref_parent(storage, SolidRefId::from_ref_id(reference), |e| {
            e.parent_comp_solid_id
        }),
        RefKind::CHILD => ref_parent(storage, ChildRefId::from_ref_id(reference), |e| {
            e.parent_compound_id
        }),
        RefKind::OCCURRENCE => ref_parent(storage, OccurrenceRefId::from_ref_id(reference), |e| {
            e.parent_product_id
        }),
        _ => NodeId::default(),
    }
}

// The parent node of a reference in range (invalid otherwise).
fn ref_parent<T: RefTag, P: Into<NodeId>>(
    storage: &crate::brep_graph_inc::Storage,
    reference: TypedId<T>,
    parent: impl Fn(&T::Entry) -> P,
) -> NodeId {
    if !reference.is_valid_for(storage.nb::<T>()) {
        return NodeId::default();
    }
    let node: NodeId = parent(storage.entry(reference)).into();
    // The native parent check is `IsValid(Nb...)` on the typed parent id.
    if node.is_valid() && node.visit(InRange(storage)) {
        node
    } else {
        NodeId::default()
    }
}

struct InRange<'s>(&'s crate::brep_graph_inc::Storage);

impl NodeVisitor for InRange<'_> {
    type Output = bool;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> bool {
        id.is_valid_for(self.0.nb::<T>())
    }
}

// The wires of the active coedges of an edge (active wires only).
fn edge_parent_wires(storage: &crate::brep_graph_inc::Storage, edge: EdgeId) -> Vec<NodeId> {
    storage
        .edge_relations(edge)
        .co_edge_ids
        .iter()
        .filter(|co_edge| {
            co_edge.is_valid_for(storage.nb_co_edges()) && !storage.is_removed(**co_edge)
        })
        .map(|co_edge| storage.co_edge(*co_edge).parent_wire_id)
        .filter(|wire| wire.is_valid_for(storage.nb_wires()) && !storage.is_removed(*wire))
        .map(NodeId::from)
        .collect()
}

// The wire of an active coedge (in range; removal is not checked natively).
fn co_edge_parent_wire(storage: &crate::brep_graph_inc::Storage, co_edge: CoEdgeId) -> Vec<NodeId> {
    if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
        return Vec::new();
    }
    let wire = storage.co_edge(co_edge).parent_wire_id;
    if wire.is_valid_for(storage.nb_wires()) {
        vec![wire.into()]
    } else {
        Vec::new()
    }
}

// The parents of the active incoming references in range.
fn active_ref_parents<T: RefTag>(
    storage: &crate::brep_graph_inc::Storage,
    references: &[TypedId<T>],
    parent: impl Fn(&T::Entry) -> NodeId,
) -> Vec<NodeId> {
    references
        .iter()
        .filter(|id| id.is_valid_for(storage.nb::<T>()) && !storage.is_removed(**id))
        .map(|id| parent(storage.entry(*id)))
        .collect()
}

// The active parent products of the active incoming occurrence references.
fn occurrence_parent_products(
    storage: &crate::brep_graph_inc::Storage,
    occurrence: OccurrenceId,
) -> Vec<NodeId> {
    storage
        .occurrence_relations(occurrence)
        .parent_occurrence_ref_ids
        .iter()
        .filter(|id| id.is_valid_for(storage.nb_occurrence_refs()) && !storage.is_removed(**id))
        .map(|id| storage.occurrence_ref(*id).parent_product_id)
        .filter(|product| {
            product.is_valid_for(storage.nb_products()) && !storage.is_removed(*product)
        })
        .map(NodeId::from)
        .collect()
}

// The active occurrences that name a topology or product node as their child.
fn occurrences_of_node(storage: &crate::brep_graph_inc::Storage, node: NodeId) -> Vec<NodeId> {
    if !NodeKind::is_topology_kind(node.kind) && node.kind != NodeKind::PRODUCT {
        return Vec::new();
    }
    storage
        .occurrence_refs_of_node(node)
        .iter()
        .filter(|id| id.is_valid_for(storage.nb_occurrence_refs()) && !storage.is_removed(**id))
        .map(|id| storage.occurrence_ref(*id).child_occurrence_id)
        .filter(|occurrence| {
            occurrence.is_valid_for(storage.nb_occurrences()) && !storage.is_removed(*occurrence)
        })
        .map(NodeId::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph_inc::ParityOrientation;

    #[test]
    fn modification_propagates_subtree_generations() {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let (shell, face) = (storage.append_shell(), storage.append_face());
        let face_ref = storage.attach_face_to_shell(shell, face, ParityOrientation::FORWARD);
        graph.mark_modified(face.into());
        let face_def = &graph.inc_storage().face(face).base;
        assert_eq!((face_def.own_gen, face_def.subtree_gen), (1, 1));
        let shell_def = &graph.inc_storage().shell(shell).base;
        assert_eq!((shell_def.own_gen, shell_def.subtree_gen), (0, 1));
        graph.mark_ref_modified(face_ref.into());
        assert_eq!(graph.inc_storage().shell(shell).base.own_gen, 1);
        graph.inc_storage().set_deferred_mode(true);
        graph.mark_modified(face.into());
        assert_eq!(
            graph.inc_storage().deferred_modified(),
            &[NodeId::from(face)]
        );
    }
}
