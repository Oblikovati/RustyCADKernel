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
// Rust translation of the rebind members of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The rebind members: they move the incoming relation entries of a use from an old
//! child to a new one after the caller changed (or before it changes) the use's target.
use super::Storage;
use super::relation_sets::{append_unique, erase_first};
use crate::brep_graph::*;

impl Storage {
    /// `RebindOccurrenceChild(theOccurrence, theOldChild, theNewChild)`: every active
    /// reference to the occurrence moves from the old child's incoming list to the new.
    pub fn rebind_occurrence_child(
        &mut self,
        occurrence: OccurrenceId,
        old_child: NodeId,
        new_child: NodeId,
    ) {
        if old_child == new_child || !occurrence.is_valid_for(self.nb_occurrences()) {
            return;
        }
        for index in 0..self.nb_occurrence_refs() {
            let ref_id = OccurrenceRefId::new(index);
            if self.is_removed(ref_id)
                || self.occurrence_ref(ref_id).child_occurrence_id != occurrence
            {
                continue;
            }
            if old_child.is_valid() {
                self.forget_occurrence_use(old_child, ref_id);
            }
            if new_child.is_valid() {
                append_unique(
                    self.change_occurrence_refs_of_node_internal(new_child),
                    ref_id,
                );
                self.set_has_occurrence_parent(new_child, true);
            }
        }
    }

    /// `RebindVertexEdge(theOldVertex, theNewVertex, theEdge, theExcludingRef)`: the
    /// edge's active vertex references (but the excluded one) that name the old vertex
    /// now name the new one, and the vertex-edge lists follow.
    pub fn rebind_vertex_edge(
        &mut self,
        old_vertex: VertexId,
        new_vertex: VertexId,
        edge: EdgeId,
        excluding: VertexRefId,
    ) {
        if old_vertex == new_vertex || !edge.is_valid_for(self.nb_edges()) || self.is_removed(edge)
        {
            return;
        }
        let (start, end) = self.edge_vertex_refs(edge);
        self.retarget_vertex_ref(start, excluding, old_vertex, new_vertex);
        self.retarget_vertex_ref(end, excluding, old_vertex, new_vertex);
        let names_old = |s: &Self| {
            s.ref_targets(start, excluding, old_vertex) || s.ref_targets(end, excluding, old_vertex)
        };
        if old_vertex.is_valid_for(self.nb_vertices()) && !names_old(self) {
            erase_first(
                &mut self.change_vertex_relations_internal(old_vertex).edge_ids,
                edge,
            );
        }
        let names_new = self.ref_targets(start, excluding, new_vertex)
            || self.ref_targets(end, excluding, new_vertex);
        if new_vertex.is_valid_for(self.nb_vertices()) && names_new {
            append_unique(
                &mut self.change_vertex_relations_internal(new_vertex).edge_ids,
                edge,
            );
        }
    }

    /// `RebindVertexRef(theRefId, theOldVertex, theNewVertex)`: the reference (when it
    /// names the old vertex and its edge is active) names the new vertex.
    pub fn rebind_vertex_ref(
        &mut self,
        ref_id: VertexRefId,
        old_vertex: VertexId,
        new_vertex: VertexId,
    ) {
        if old_vertex == new_vertex || !ref_id.is_valid_for(self.nb_vertex_refs()) {
            return;
        }
        let entry = self.vertex_ref(ref_id).clone();
        if entry.child_vertex_id != old_vertex {
            return;
        }
        let edge = entry.parent_edge_id;
        if !edge.is_valid_for(self.nb_edges()) || self.is_removed(edge) {
            return;
        }
        self.change_vertex_ref(ref_id).child_vertex_id = new_vertex;
        let (start, end) = self.edge_vertex_refs(edge);
        let other_names_old = self.ref_targets(start, ref_id, old_vertex)
            || self.ref_targets(end, ref_id, old_vertex);
        if old_vertex.is_valid_for(self.nb_vertices()) && !other_names_old {
            erase_first(
                &mut self.change_vertex_relations_internal(old_vertex).edge_ids,
                edge,
            );
        }
        if new_vertex.is_valid_for(self.nb_vertices()) {
            append_unique(
                &mut self.change_vertex_relations_internal(new_vertex).edge_ids,
                edge,
            );
        }
    }

    /// `RebindCoEdgeEdge(theCoEdge, theOldEdge, theNewEdge)`.
    pub fn rebind_co_edge_edge(&mut self, co_edge: CoEdgeId, old_edge: EdgeId, new_edge: EdgeId) {
        if old_edge == new_edge || !co_edge.is_valid_for(self.nb_co_edges()) {
            return;
        }
        if old_edge.is_valid_for(self.nb_edges()) {
            erase_first(
                &mut self.change_edge_relations_internal(old_edge).co_edge_ids,
                co_edge,
            );
        }
        if new_edge.is_valid_for(self.nb_edges()) {
            append_unique(
                &mut self.change_edge_relations_internal(new_edge).co_edge_ids,
                co_edge,
            );
        }
    }

    /// `RebindWireRef(theRefId, theOldWire, theNewWire)`: the old wire loses its face
    /// context when it keeps no use on the reference's face.
    pub fn rebind_wire_ref(&mut self, ref_id: WireRefId, old_wire: WireId, new_wire: WireId) {
        if old_wire == new_wire || !ref_id.is_valid_for(self.nb_wire_refs()) {
            return;
        }
        let face = self.wire_ref(ref_id).parent_face_id;
        if old_wire.is_valid_for(self.nb_wires()) {
            erase_first(
                &mut self
                    .change_wire_relations_internal(old_wire)
                    .parent_wire_ref_ids,
                ref_id,
            );
            if face.is_valid_for(self.nb_faces()) && !self.wire_has_face_use(old_wire, face) {
                self.clear_wire_face_context(old_wire, face);
            }
        }
        if new_wire.is_valid_for(self.nb_wires()) && !self.is_removed(ref_id) {
            append_unique(
                &mut self
                    .change_wire_relations_internal(new_wire)
                    .parent_wire_ref_ids,
                ref_id,
            );
        }
    }

    /// `RebindFaceRef(theRefId, theOldFace, theNewFace)`.
    pub fn rebind_face_ref(&mut self, ref_id: FaceRefId, old_face: FaceId, new_face: FaceId) {
        if old_face == new_face || !ref_id.is_valid_for(self.nb_face_refs()) {
            return;
        }
        if old_face.is_valid_for(self.nb_faces()) {
            erase_first(
                &mut self
                    .change_face_relations_internal(old_face)
                    .parent_face_ref_ids,
                ref_id,
            );
        }
        if new_face.is_valid_for(self.nb_faces()) && !self.is_removed(ref_id) {
            let incoming = &mut self
                .change_face_relations_internal(new_face)
                .parent_face_ref_ids;
            append_unique(incoming, ref_id);
        }
    }

    /// `RebindShellRef(theRefId, theOldShell, theNewShell)`.
    pub fn rebind_shell_ref(&mut self, ref_id: ShellRefId, old_shell: ShellId, new_shell: ShellId) {
        if old_shell == new_shell || !ref_id.is_valid_for(self.nb_shell_refs()) {
            return;
        }
        if old_shell.is_valid_for(self.nb_shells()) {
            let incoming = &mut self
                .change_shell_relations_internal(old_shell)
                .parent_shell_ref_ids;
            erase_first(incoming, ref_id);
        }
        if new_shell.is_valid_for(self.nb_shells()) && !self.is_removed(ref_id) {
            let incoming = &mut self
                .change_shell_relations_internal(new_shell)
                .parent_shell_ref_ids;
            append_unique(incoming, ref_id);
        }
    }

    /// `RebindSolidRef(theRefId, theOldSolid, theNewSolid)`.
    pub fn rebind_solid_ref(&mut self, ref_id: SolidRefId, old_solid: SolidId, new_solid: SolidId) {
        if old_solid == new_solid || !ref_id.is_valid_for(self.nb_solid_refs()) {
            return;
        }
        if old_solid.is_valid_for(self.nb_solids()) {
            let incoming = &mut self
                .change_solid_relations_internal(old_solid)
                .parent_solid_ref_ids;
            erase_first(incoming, ref_id);
        }
        if new_solid.is_valid_for(self.nb_solids()) && !self.is_removed(ref_id) {
            let incoming = &mut self
                .change_solid_relations_internal(new_solid)
                .parent_solid_ref_ids;
            append_unique(incoming, ref_id);
        }
    }

    /// `RebindChildRef(theRefId, theOldChild, theNewChild)`.
    pub fn rebind_child_ref(&mut self, ref_id: ChildRefId, old_child: NodeId, new_child: NodeId) {
        if old_child == new_child || !ref_id.is_valid_for(self.nb_child_refs()) {
            return;
        }
        if old_child.is_valid() {
            self.forget_compound_use(old_child, ref_id);
        }
        if new_child.is_valid() && !self.is_removed(ref_id) {
            append_unique(
                self.change_compound_refs_of_node_internal(new_child),
                ref_id,
            );
            self.set_has_compound_parent(new_child, true);
        }
    }

    /// `RebindOccurrenceRef(theRefId, theOldOccurrence, theNewOccurrence)`.
    pub fn rebind_occurrence_ref(
        &mut self,
        ref_id: OccurrenceRefId,
        old_occurrence: OccurrenceId,
        new_occurrence: OccurrenceId,
    ) {
        if old_occurrence == new_occurrence || !ref_id.is_valid_for(self.nb_occurrence_refs()) {
            return;
        }
        let old_child = self.occurrence_child(old_occurrence);
        let new_child = self.occurrence_child(new_occurrence);
        if old_child.is_valid() {
            self.forget_occurrence_use(old_child, ref_id);
        }
        if old_occurrence.is_valid_for(self.nb_occurrences()) {
            let incoming = &mut self
                .change_occurrence_relations_internal(old_occurrence)
                .parent_occurrence_ref_ids;
            erase_first(incoming, ref_id);
        }
        let active = !self.is_removed(ref_id);
        if new_occurrence.is_valid_for(self.nb_occurrences()) && active {
            let incoming = &mut self
                .change_occurrence_relations_internal(new_occurrence)
                .parent_occurrence_ref_ids;
            append_unique(incoming, ref_id);
        }
        if new_child.is_valid() && active {
            append_unique(
                self.change_occurrence_refs_of_node_internal(new_child),
                ref_id,
            );
            self.set_has_occurrence_parent(new_child, true);
        }
    }

    // The child node of an occurrence in range, invalid otherwise.
    fn occurrence_child(&self, occurrence: OccurrenceId) -> NodeId {
        if occurrence.is_valid_for(self.nb_occurrences()) {
            return self.occurrence(occurrence).child_node_id;
        }
        NodeId::default()
    }

    fn edge_vertex_refs(&self, edge: EdgeId) -> (VertexRefId, VertexRefId) {
        let def = self.edge(edge);
        (def.start_vertex_ref_id, def.end_vertex_ref_id)
    }

    // `refTargets`: an active reference in range, other than the excluded one, that
    // names the vertex.
    fn ref_targets(&self, ref_id: VertexRefId, excluding: VertexRefId, vertex: VertexId) -> bool {
        if !ref_id.is_valid_for(self.nb_vertex_refs())
            || ref_id == excluding
            || self.is_removed(ref_id)
        {
            return false;
        }
        self.vertex_ref(ref_id).child_vertex_id == vertex
    }

    // `rebindRef` of `RebindVertexEdge`.
    fn retarget_vertex_ref(
        &mut self,
        ref_id: VertexRefId,
        excluding: VertexRefId,
        old_vertex: VertexId,
        new_vertex: VertexId,
    ) {
        if !ref_id.is_valid_for(self.nb_vertex_refs())
            || ref_id == excluding
            || self.is_removed(ref_id)
        {
            return;
        }
        let entry = self.change_vertex_ref(ref_id);
        if entry.child_vertex_id == old_vertex {
            entry.child_vertex_id = new_vertex;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph_inc::ParityOrientation;

    fn edge_with_vertices(storage: &mut Storage) -> (EdgeId, VertexId, VertexRefId) {
        let (edge, vertex) = (storage.append_edge(), storage.append_vertex());
        let start = storage.append_vertex_ref();
        let entry = storage.change_vertex_ref(start);
        entry.child_vertex_id = vertex;
        entry.parent_edge_id = edge;
        storage.change_edge(edge).start_vertex_ref_id = start;
        storage.attach_edge_to_vertex(edge, vertex);
        (edge, vertex, start)
    }

    #[test]
    fn vertex_rebinds_move_the_edge_lists() {
        let mut storage = Storage::new();
        let (edge, old_vertex, start) = edge_with_vertices(&mut storage);
        let new_vertex = storage.append_vertex();
        storage.rebind_vertex_ref(start, old_vertex, new_vertex);
        assert_eq!(storage.vertex_ref(start).child_vertex_id, new_vertex);
        assert!(storage.vertex_relations(old_vertex).edge_ids.is_empty());
        assert_eq!(storage.vertex_relations(new_vertex).edge_ids, vec![edge]);
        storage.rebind_vertex_edge(new_vertex, old_vertex, edge, VertexRefId::default());
        assert_eq!(storage.vertex_ref(start).child_vertex_id, old_vertex);
        assert_eq!(storage.vertex_relations(old_vertex).edge_ids, vec![edge]);
    }

    #[test]
    fn ref_rebinds_move_incoming_lists() {
        let mut storage = Storage::new();
        let (shell, face, other) = (
            storage.append_shell(),
            storage.append_face(),
            storage.append_face(),
        );
        let face_ref = storage.attach_face_to_shell(shell, face, ParityOrientation::FORWARD);
        storage.rebind_face_ref(face_ref, face, other);
        assert!(storage.face_relations(face).parent_face_ref_ids.is_empty());
        assert_eq!(
            storage.face_relations(other).parent_face_ref_ids,
            vec![face_ref]
        );
        let compound = storage.append_compound();
        let child = storage.attach_child_to_compound(
            compound,
            face.into(),
            &Default::default(),
            ParityOrientation::FORWARD,
        );
        storage.rebind_child_ref(child, face.into(), other.into());
        assert!(!storage.has_compound_parent(face.into()));
        assert!(storage.has_compound_parent(other.into()));
        let (edge, new_edge) = (storage.append_edge(), storage.append_edge());
        let wire = storage.append_wire();
        let co_edge = storage.create_co_edge_use(wire, edge, face, ParityOrientation::FORWARD);
        storage.rebind_co_edge_edge(co_edge, edge, new_edge);
        assert_eq!(storage.edge_relations(new_edge).co_edge_ids, vec![co_edge]);
    }
}
