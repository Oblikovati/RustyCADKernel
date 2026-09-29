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
// Rust translation of the attach members of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The incremental attach members: each creates one coedge or reference and updates the
//! forward list of the parent and the incoming list of the child.
use super::Storage;
use super::relation_sets::append_unique;
use crate::brep_graph::*;
use crate::brep_graph_inc::ParityOrientation;
use rustycad_tkmath::toploc::Location;

impl Storage {
    /// `CreateCoEdgeUse(theParentWireId, theChildEdgeId, theFaceId, theOrientation)`: a
    /// new coedge appended to the wire and registered on the edge. The wire and edge
    /// relation slots must exist (undefined natively otherwise; a panic here).
    pub fn create_co_edge_use(
        &mut self,
        parent_wire: WireId,
        child_edge: EdgeId,
        face: FaceId,
        orientation: ParityOrientation,
    ) -> CoEdgeId {
        let co_edge_id = self.append_co_edge();
        let co_edge = self.change_co_edge(co_edge_id);
        co_edge.parent_wire_id = parent_wire;
        co_edge.child_edge_id = child_edge;
        co_edge.face_id = face;
        co_edge.orientation = orientation;
        self.change_wire_relations_internal(parent_wire)
            .co_edge_ids
            .push(co_edge_id);
        append_unique(
            &mut self.change_edge_relations_internal(child_edge).co_edge_ids,
            co_edge_id,
        );
        co_edge_id
    }

    /// `AttachEdgeToVertex(theEdgeId, theVertexId)`: records the edge on the vertex; ids
    /// out of range are ignored.
    pub fn attach_edge_to_vertex(&mut self, edge: EdgeId, vertex: VertexId) {
        if !edge.is_valid_for(self.nb_edges()) || !vertex.is_valid_for(self.nb_vertices()) {
            return;
        }
        append_unique(
            &mut self.change_vertex_relations_internal(vertex).edge_ids,
            edge,
        );
    }

    /// `AttachWireToFace(theParentFaceId, theChildWireId, theOrientation)`.
    pub fn attach_wire_to_face(
        &mut self,
        parent_face: FaceId,
        child_wire: WireId,
        orientation: ParityOrientation,
    ) -> WireRefId {
        let ref_id = self.append_wire_ref();
        let entry = self.change_wire_ref(ref_id);
        entry.parent_face_id = parent_face;
        entry.child_wire_id = child_wire;
        entry.orientation = orientation;
        self.change_face_relations_internal(parent_face)
            .wire_ref_ids
            .push(ref_id);
        append_unique(
            &mut self
                .change_wire_relations_internal(child_wire)
                .parent_wire_ref_ids,
            ref_id,
        );
        ref_id
    }

    /// `AttachFaceToShell(theParentShellId, theChildFaceId, theOrientation)`.
    pub fn attach_face_to_shell(
        &mut self,
        parent_shell: ShellId,
        child_face: FaceId,
        orientation: ParityOrientation,
    ) -> FaceRefId {
        let ref_id = self.append_face_ref();
        let entry = self.change_face_ref(ref_id);
        entry.parent_shell_id = parent_shell;
        entry.child_face_id = child_face;
        entry.orientation = orientation;
        self.change_shell_relations_internal(parent_shell)
            .face_ref_ids
            .push(ref_id);
        append_unique(
            &mut self
                .change_face_relations_internal(child_face)
                .parent_face_ref_ids,
            ref_id,
        );
        ref_id
    }

    /// `AttachShellToSolid(theParentSolidId, theChildShellId, theOrientation)`.
    pub fn attach_shell_to_solid(
        &mut self,
        parent_solid: SolidId,
        child_shell: ShellId,
        orientation: ParityOrientation,
    ) -> ShellRefId {
        let ref_id = self.append_shell_ref();
        let entry = self.change_shell_ref(ref_id);
        entry.parent_solid_id = parent_solid;
        entry.child_shell_id = child_shell;
        entry.orientation = orientation;
        self.change_solid_relations_internal(parent_solid)
            .shell_ref_ids
            .push(ref_id);
        append_unique(
            &mut self
                .change_shell_relations_internal(child_shell)
                .parent_shell_ref_ids,
            ref_id,
        );
        ref_id
    }

    /// `AttachSolidToCompSolid(theParentCompSolidId, theChildSolidId, theOrientation)`.
    pub fn attach_solid_to_comp_solid(
        &mut self,
        parent_comp_solid: CompSolidId,
        child_solid: SolidId,
        orientation: ParityOrientation,
    ) -> SolidRefId {
        let ref_id = self.append_solid_ref();
        let entry = self.change_solid_ref(ref_id);
        entry.parent_comp_solid_id = parent_comp_solid;
        entry.child_solid_id = child_solid;
        entry.orientation = orientation;
        self.change_comp_solid_relations_internal(parent_comp_solid)
            .solid_ref_ids
            .push(ref_id);
        append_unique(
            &mut self
                .change_solid_relations_internal(child_solid)
                .parent_solid_ref_ids,
            ref_id,
        );
        ref_id
    }

    /// `AttachChildToCompound(theParentCompoundId, theChildNodeId, theLocation,
    /// theOrientation)`: also records the compound parent of a valid child.
    pub fn attach_child_to_compound(
        &mut self,
        parent_compound: CompoundId,
        child: NodeId,
        location: &Location,
        orientation: ParityOrientation,
    ) -> ChildRefId {
        let ref_id = self.append_child_ref();
        let entry = self.change_child_ref(ref_id);
        entry.parent_compound_id = parent_compound;
        entry.child_node_id = child;
        entry.local_location = location.clone();
        entry.orientation = orientation;
        self.change_compound_relations_internal(parent_compound)
            .child_ref_ids
            .push(ref_id);
        if child.is_valid() {
            append_unique(self.change_compound_refs_of_node_internal(child), ref_id);
            self.set_has_compound_parent(child, true);
        }
        ref_id
    }

    /// `AttachOccurrenceToProduct(theParentProductId, theChildOccurrenceId,
    /// theLocation)`: also records the occurrence parent of the occurrence's child.
    pub fn attach_occurrence_to_product(
        &mut self,
        parent_product: ProductId,
        child_occurrence: OccurrenceId,
        location: &Location,
    ) -> OccurrenceRefId {
        let ref_id = self.append_occurrence_ref();
        let entry = self.change_occurrence_ref(ref_id);
        entry.parent_product_id = parent_product;
        entry.child_occurrence_id = child_occurrence;
        entry.local_location = location.clone();
        self.change_product_relations_internal(parent_product)
            .occurrence_ref_ids
            .push(ref_id);
        if child_occurrence.is_valid_for(self.nb_occurrences()) {
            self.register_occurrence_parent(child_occurrence, ref_id);
        }
        ref_id
    }

    // The incoming part of `AttachOccurrenceToProduct` for an occurrence in range.
    fn register_occurrence_parent(&mut self, occurrence: OccurrenceId, ref_id: OccurrenceRefId) {
        append_unique(
            &mut self
                .change_occurrence_relations_internal(occurrence)
                .parent_occurrence_ref_ids,
            ref_id,
        );
        let child = self.occurrence(occurrence).child_node_id;
        if child.is_valid() {
            append_unique(self.change_occurrence_refs_of_node_internal(child), ref_id);
            self.set_has_occurrence_parent(child, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_members_update_both_directions() {
        let mut storage = Storage::new();
        let (face, wire, edge) = (
            storage.append_face(),
            storage.append_wire(),
            storage.append_edge(),
        );
        let vertex = storage.append_vertex();
        let co_edge = storage.create_co_edge_use(wire, edge, face, ParityOrientation::REVERSED);
        assert_eq!(storage.wire_relations(wire).co_edge_ids, vec![co_edge]);
        assert_eq!(storage.edge_relations(edge).co_edge_ids, vec![co_edge]);
        storage.attach_edge_to_vertex(edge, vertex);
        storage.attach_edge_to_vertex(edge, vertex);
        assert_eq!(storage.vertex_relations(vertex).edge_ids, vec![edge]);
        let wire_ref = storage.attach_wire_to_face(face, wire, ParityOrientation::FORWARD);
        assert_eq!(
            storage.wire_relations(wire).parent_wire_ref_ids,
            vec![wire_ref]
        );
        let compound = storage.append_compound();
        let child = storage.attach_child_to_compound(
            compound,
            face.into(),
            &Location::default(),
            ParityOrientation::FORWARD,
        );
        assert_eq!(storage.compound_refs_of_node(face.into()), &[child]);
        assert!(storage.has_compound_parent(face.into()));
        let (product, occurrence) = (storage.append_product(), storage.append_occurrence());
        storage.change_occurrence(occurrence).child_node_id = compound.into();
        let occurrence_ref =
            storage.attach_occurrence_to_product(product, occurrence, &Location::default());
        assert_eq!(
            storage.occurrence_refs_of_node(compound.into()),
            &[occurrence_ref]
        );
        assert!(storage.has_occurrence_parent(compound.into()));
    }
}
