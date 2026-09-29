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
// Rust translation of rebuildDerivedRelationsInternal of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The whole-table relation rebuild (raw load, compact, explicit repair).
use super::Storage;
use crate::brep_graph::*;

/// `prepareRelationTable`: a zero count releases the table; a shorter table grows.
pub(crate) fn prepare_relation_table<R: Default>(table: &mut Vec<R>, count: u32) {
    if count == 0 {
        *table = Vec::new();
        return;
    }
    if table.len() < count as usize {
        table.resize_with(count as usize, R::default);
    }
}

impl Storage {
    /// `RebuildDerivedRelations()`: rebuilds every incoming list from the forward lists
    /// and the record endpoints, then recounts the active slots.
    pub fn rebuild_derived_relations(&mut self) {
        self.rebuild_derived_relations_internal(true);
    }

    /// `RebuildDerivedRelationsPreservingActiveCounts()`.
    pub fn rebuild_derived_relations_preserving_active_counts(&mut self) {
        self.rebuild_derived_relations_internal(false);
    }

    fn rebuild_derived_relations_internal(&mut self, recount_active_counts: bool) {
        self.prepare_relation_tables();
        self.clear_incoming_relations();
        self.clear_parent_flags();
        self.prune_forward_lists();
        self.rebuild_vertex_edges();
        self.rebuild_edge_co_edges();
        self.rebuild_ref_incoming_lists();
        self.rebuild_sparse_incoming_maps();
        self.rebuild_parent_flags();
        if recount_active_counts {
            self.recount_active_counts();
        }
    }

    pub(super) fn prepare_relation_tables(&mut self) {
        prepare_relation_table(&mut self.vertex_relations, self.vertices.nb());
        prepare_relation_table(&mut self.edge_relations, self.edges.nb());
        prepare_relation_table(&mut self.wire_relations, self.wires.nb());
        prepare_relation_table(&mut self.face_relations, self.faces.nb());
        prepare_relation_table(&mut self.shell_relations, self.shells.nb());
        prepare_relation_table(&mut self.solid_relations, self.solids.nb());
        prepare_relation_table(&mut self.compound_relations, self.compounds.nb());
        prepare_relation_table(&mut self.comp_solid_relations, self.comp_solids.nb());
        prepare_relation_table(&mut self.product_relations, self.products.nb());
        prepare_relation_table(&mut self.occurrence_relations, self.occurrences.nb());
    }

    fn clear_incoming_relations(&mut self) {
        let (nb_v, nb_e, nb_w) = (self.nb_vertices(), self.nb_edges(), self.nb_wires());
        self.vertex_relations[..nb_v as usize]
            .iter_mut()
            .for_each(|r| r.edge_ids.clear());
        self.edge_relations[..nb_e as usize]
            .iter_mut()
            .for_each(|r| r.co_edge_ids.clear());
        let wires = &mut self.wire_relations[..nb_w as usize];
        wires.iter_mut().for_each(|r| r.parent_wire_ref_ids.clear());
        let faces = &mut self.face_relations[..self.faces.nb() as usize];
        faces.iter_mut().for_each(|r| r.parent_face_ref_ids.clear());
        let shells = &mut self.shell_relations[..self.shells.nb() as usize];
        shells
            .iter_mut()
            .for_each(|r| r.parent_shell_ref_ids.clear());
        let solids = &mut self.solid_relations[..self.solids.nb() as usize];
        solids
            .iter_mut()
            .for_each(|r| r.parent_solid_ref_ids.clear());
        let occurrences = &mut self.occurrence_relations[..self.occurrences.nb() as usize];
        occurrences
            .iter_mut()
            .for_each(|r| r.parent_occurrence_ref_ids.clear());
        self.node_to_compounds.clear();
        self.node_to_occurrences.clear();
    }

    fn clear_parent_flags(&mut self) {
        macro_rules! clear {
            ($($store:ident),*) => {$(
                self.$store.has_compound_parent.clear_all();
                self.$store.has_occurrence_parent.clear_all();
            )*};
        }
        clear!(
            vertices,
            edges,
            co_edges,
            wires,
            faces,
            shells,
            solids,
            compounds,
            comp_solids,
            products,
            occurrences
        );
    }

    // The `pruneRelationIds` passes: forward lists keep only active entries that name
    // their owner back.
    fn prune_forward_lists(&mut self) {
        for index in 0..self.nb_wires() {
            let wire = WireId::new(index);
            let mut ids = std::mem::take(&mut self.wire_relations[index as usize].co_edge_ids);
            ids.retain(|id| {
                !self.is_removed(wire)
                    && id.is_valid_for(self.nb_co_edges())
                    && !self.is_removed(*id)
                    && self.co_edge(*id).parent_wire_id == wire
            });
            self.wire_relations[index as usize].co_edge_ids = ids;
        }
        self.prune_face_wire_refs();
        self.prune_shell_face_refs();
        self.prune_solid_shell_refs();
        self.prune_comp_solid_solid_refs();
        self.prune_compound_child_refs();
        self.prune_product_occurrence_refs();
    }

    fn prune_face_wire_refs(&mut self) {
        for index in 0..self.nb_faces() {
            let face = FaceId::new(index);
            let mut ids = std::mem::take(&mut self.face_relations[index as usize].wire_ref_ids);
            ids.retain(|id| {
                !self.is_removed(face)
                    && id.is_valid_for(self.nb_wire_refs())
                    && !self.is_removed(*id)
                    && self.wire_ref(*id).parent_face_id == face
            });
            self.face_relations[index as usize].wire_ref_ids = ids;
        }
    }

    fn prune_shell_face_refs(&mut self) {
        for index in 0..self.nb_shells() {
            let shell = ShellId::new(index);
            let mut ids = std::mem::take(&mut self.shell_relations[index as usize].face_ref_ids);
            ids.retain(|id| {
                !self.is_removed(shell)
                    && id.is_valid_for(self.nb_face_refs())
                    && !self.is_removed(*id)
                    && self.face_ref(*id).parent_shell_id == shell
            });
            self.shell_relations[index as usize].face_ref_ids = ids;
        }
    }

    fn prune_solid_shell_refs(&mut self) {
        for index in 0..self.nb_solids() {
            let solid = SolidId::new(index);
            let mut ids = std::mem::take(&mut self.solid_relations[index as usize].shell_ref_ids);
            ids.retain(|id| {
                !self.is_removed(solid)
                    && id.is_valid_for(self.nb_shell_refs())
                    && !self.is_removed(*id)
                    && self.shell_ref(*id).parent_solid_id == solid
            });
            self.solid_relations[index as usize].shell_ref_ids = ids;
        }
    }

    fn prune_comp_solid_solid_refs(&mut self) {
        for index in 0..self.nb_comp_solids() {
            let comp_solid = CompSolidId::new(index);
            let slot = index as usize;
            let mut ids = std::mem::take(&mut self.comp_solid_relations[slot].solid_ref_ids);
            ids.retain(|id| {
                !self.is_removed(comp_solid)
                    && id.is_valid_for(self.nb_solid_refs())
                    && !self.is_removed(*id)
                    && self.solid_ref(*id).parent_comp_solid_id == comp_solid
            });
            self.comp_solid_relations[slot].solid_ref_ids = ids;
        }
    }

    fn prune_compound_child_refs(&mut self) {
        for index in 0..self.nb_compounds() {
            let compound = CompoundId::new(index);
            let slot = index as usize;
            let mut ids = std::mem::take(&mut self.compound_relations[slot].child_ref_ids);
            ids.retain(|id| {
                !self.is_removed(compound)
                    && id.is_valid_for(self.nb_child_refs())
                    && !self.is_removed(*id)
                    && self.child_ref(*id).parent_compound_id == compound
            });
            self.compound_relations[slot].child_ref_ids = ids;
        }
    }

    fn prune_product_occurrence_refs(&mut self) {
        for index in 0..self.nb_products() {
            let product = ProductId::new(index);
            let slot = index as usize;
            let mut ids = std::mem::take(&mut self.product_relations[slot].occurrence_ref_ids);
            ids.retain(|id| {
                !self.is_removed(product)
                    && id.is_valid_for(self.nb_occurrence_refs())
                    && !self.is_removed(*id)
                    && self.occurrence_ref(*id).parent_product_id == product
            });
            self.product_relations[slot].occurrence_ref_ids = ids;
        }
    }

    // The vertex-edge lists from the active edges' active vertex references.
    fn rebuild_vertex_edges(&mut self) {
        for index in 0..self.nb_edges() {
            let edge = EdgeId::new(index);
            if self.is_removed(edge) {
                continue;
            }
            let (start, end) = (
                self.edge(edge).start_vertex_ref_id,
                self.edge(edge).end_vertex_ref_id,
            );
            for ref_id in [start, end] {
                if let Some(vertex) = self.active_vertex_of(ref_id) {
                    self.change_vertex_relations_internal(vertex)
                        .edge_ids
                        .push(edge);
                }
            }
        }
    }

    fn active_vertex_of(&self, ref_id: VertexRefId) -> Option<VertexId> {
        if !ref_id.is_valid_for(self.nb_vertex_refs()) || self.is_removed(ref_id) {
            return None;
        }
        let vertex = self.vertex_ref(ref_id).child_vertex_id;
        (vertex.is_valid_for(self.nb_vertices()) && !self.is_removed(vertex)).then_some(vertex)
    }

    // The edge-coedge lists from the active coedges.
    fn rebuild_edge_co_edges(&mut self) {
        for index in 0..self.nb_co_edges() {
            let co_edge = CoEdgeId::new(index);
            if self.is_removed(co_edge) {
                continue;
            }
            let edge = self.co_edge(co_edge).child_edge_id;
            if edge.is_valid_for(self.nb_edges()) && !self.is_removed(edge) {
                self.change_edge_relations_internal(edge)
                    .co_edge_ids
                    .push(co_edge);
            }
        }
    }

    // The incoming reference lists of wires, faces, shells and solids.
    fn rebuild_ref_incoming_lists(&mut self) {
        for index in 0..self.nb_faces() {
            if self.is_removed(FaceId::new(index)) {
                continue;
            }
            for ref_id in self.face_relations[index as usize].wire_ref_ids.clone() {
                if let Some(wire) = self.active_child(ref_id, |s, r| s.wire_ref(r).child_wire_id) {
                    self.change_wire_relations_internal(wire)
                        .parent_wire_ref_ids
                        .push(ref_id);
                }
            }
        }
        for index in 0..self.nb_shells() {
            if self.is_removed(ShellId::new(index)) {
                continue;
            }
            for ref_id in self.shell_relations[index as usize].face_ref_ids.clone() {
                if let Some(face) = self.active_child(ref_id, |s, r| s.face_ref(r).child_face_id) {
                    self.change_face_relations_internal(face)
                        .parent_face_ref_ids
                        .push(ref_id);
                }
            }
        }
        self.rebuild_shell_and_solid_incoming();
    }

    fn rebuild_shell_and_solid_incoming(&mut self) {
        for index in 0..self.nb_solids() {
            if self.is_removed(SolidId::new(index)) {
                continue;
            }
            for ref_id in self.solid_relations[index as usize].shell_ref_ids.clone() {
                let child = self.active_child(ref_id, |s, r| s.shell_ref(r).child_shell_id);
                if let Some(shell) = child {
                    self.change_shell_relations_internal(shell)
                        .parent_shell_ref_ids
                        .push(ref_id);
                }
            }
        }
        for index in 0..self.nb_comp_solids() {
            if self.is_removed(CompSolidId::new(index)) {
                continue;
            }
            for ref_id in self.comp_solid_relations[index as usize]
                .solid_ref_ids
                .clone()
            {
                let child = self.active_child(ref_id, |s, r| s.solid_ref(r).child_solid_id);
                if let Some(solid) = child {
                    self.change_solid_relations_internal(solid)
                        .parent_solid_ref_ids
                        .push(ref_id);
                }
            }
        }
    }

    // The active child definition of an active reference.
    fn active_child<R, C>(
        &self,
        ref_id: TypedId<R>,
        child: fn(&Self, TypedId<R>) -> TypedId<C>,
    ) -> Option<TypedId<C>>
    where
        R: super::RefTag,
        C: super::NodeTag,
    {
        if !ref_id.is_valid_for(self.nb::<R>()) || self.is_removed(ref_id) {
            return None;
        }
        let child_id = child(self, ref_id);
        (child_id.is_valid_for(self.nb::<C>()) && !self.is_removed(child_id)).then_some(child_id)
    }

    // The compound and occurrence incoming maps and the occurrence incoming lists.
    fn rebuild_sparse_incoming_maps(&mut self) {
        for index in 0..self.nb_compounds() {
            if self.is_removed(CompoundId::new(index)) {
                continue;
            }
            for ref_id in self.compound_relations[index as usize]
                .child_ref_ids
                .clone()
            {
                if !ref_id.is_valid_for(self.nb_child_refs()) || self.is_removed(ref_id) {
                    continue;
                }
                let child = self.child_ref(ref_id).child_node_id;
                if child.is_valid() && !self.is_node_removed(child) {
                    self.change_compound_refs_of_node_internal(child)
                        .push(ref_id);
                }
            }
        }
        for index in 0..self.nb_products() {
            if self.is_removed(ProductId::new(index)) {
                continue;
            }
            for ref_id in self.product_relations[index as usize]
                .occurrence_ref_ids
                .clone()
            {
                self.rebuild_occurrence_ref(ref_id);
            }
        }
    }

    fn rebuild_occurrence_ref(&mut self, ref_id: OccurrenceRefId) {
        let child = |s: &Self, r| s.occurrence_ref(r).child_occurrence_id;
        let Some(occurrence) = self.active_child(ref_id, child) else {
            return;
        };
        let incoming = &mut self.change_occurrence_relations_internal(occurrence);
        incoming.parent_occurrence_ref_ids.push(ref_id);
        let node = self.occurrence(occurrence).child_node_id;
        if node.is_valid() && !self.is_node_removed(node) {
            self.change_occurrence_refs_of_node_internal(node)
                .push(ref_id);
        }
    }

    // The parent bits of every node with a non-empty incoming map entry.
    fn rebuild_parent_flags(&mut self) {
        let compound_children: Vec<NodeId> = self
            .node_to_compounds
            .iter()
            .filter(|(_, refs)| !refs.is_empty())
            .map(|(node, _)| *node)
            .collect();
        for node in compound_children {
            self.set_has_compound_parent(node, true);
        }
        let occurrence_children: Vec<NodeId> = self
            .node_to_occurrences
            .iter()
            .filter(|(_, refs)| !refs.is_empty())
            .map(|(node, _)| *node)
            .collect();
        for node in occurrence_children {
            self.set_has_occurrence_parent(node, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph_inc::ParityOrientation;

    #[test]
    fn rebuild_recovers_incoming_lists() {
        let mut storage = Storage::new();
        let (shell, face, wire) = (
            storage.append_shell(),
            storage.append_face(),
            storage.append_wire(),
        );
        let edge = storage.append_edge();
        storage.attach_face_to_shell(shell, face, ParityOrientation::FORWARD);
        storage.attach_wire_to_face(face, wire, ParityOrientation::FORWARD);
        storage.create_co_edge_use(wire, edge, face, ParityOrientation::FORWARD);
        let before = (
            storage.face_relations(face).clone(),
            storage.edge_relations(edge).clone(),
        );
        storage
            .change_face_relations_internal(face)
            .parent_face_ref_ids
            .clear();
        storage.rebuild_derived_relations();
        assert_eq!(storage.face_relations(face), &before.0);
        assert_eq!(storage.edge_relations(edge), &before.1);
        assert!(storage.validate_relations());
        let mut copy = Storage::new();
        copy.append_face();
        copy.copy_derived_relations_from(&storage);
        assert_eq!(copy.face_relations(face), &before.0);
        copy.copy_removed_flags_from(&storage);
        copy.recount_active_counts();
        assert_eq!(copy.nb_active_faces(), 1);
    }
}
