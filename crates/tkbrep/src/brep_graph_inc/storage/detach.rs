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
// Rust translation of the detach members and ReplaceCoEdgeUseWithPair of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The incremental detach members: each removes one use from the parent's forward list
//! and from the child's incoming list.
use super::Storage;
use super::relation_sets::{append_unique, erase_first};
use crate::brep_graph::*;

impl Storage {
    /// `DetachCoEdgeUse(theParentWireId, theCoEdgeId)`: true when the wire held it.
    pub fn detach_co_edge_use(&mut self, parent_wire: WireId, co_edge: CoEdgeId) -> bool {
        if !parent_wire.is_valid_for(self.nb_wires()) || !co_edge.is_valid_for(self.nb_co_edges()) {
            return false;
        }
        let edge = self.co_edge(co_edge).child_edge_id;
        let wire_list = &mut self.change_wire_relations_internal(parent_wire).co_edge_ids;
        if !erase_first(wire_list, co_edge) {
            return false;
        }
        if edge.is_valid_for(self.nb_edges()) {
            erase_first(
                &mut self.change_edge_relations_internal(edge).co_edge_ids,
                co_edge,
            );
        }
        true
    }

    /// `ReplaceCoEdgeUseWithPair(theParentWireId, theOldCoEdgeId, theNewFirstCoEdgeId,
    /// theNewSecondCoEdgeId)`: the old coedge's slot takes the pair, which is registered
    /// on its edges.
    pub fn replace_co_edge_use_with_pair(
        &mut self,
        parent_wire: WireId,
        old: CoEdgeId,
        first: CoEdgeId,
        second: CoEdgeId,
    ) -> bool {
        let nb_co_edges = self.nb_co_edges();
        if !parent_wire.is_valid_for(self.nb_wires())
            || [old, first, second]
                .iter()
                .any(|id| !id.is_valid_for(nb_co_edges))
        {
            return false;
        }
        let list = &self.wire_relations(parent_wire).co_edge_ids;
        let Some(position) = list.iter().position(|id| *id == old) else {
            return false;
        };
        self.change_co_edge(first).parent_wire_id = parent_wire;
        self.change_co_edge(second).parent_wire_id = parent_wire;
        let old_edge = self.co_edge(old).child_edge_id;
        if old_edge.is_valid_for(self.nb_edges()) {
            erase_first(
                &mut self.change_edge_relations_internal(old_edge).co_edge_ids,
                old,
            );
        }
        let list = &mut self.change_wire_relations_internal(parent_wire).co_edge_ids;
        list[position] = first;
        list.insert(position + 1, second);
        self.bind_co_edge_to_its_edge(first);
        self.bind_co_edge_to_its_edge(second);
        true
    }

    // `bindEdgeRelations` of `ReplaceCoEdgeUseWithPair`.
    fn bind_co_edge_to_its_edge(&mut self, co_edge: CoEdgeId) {
        let edge = self.co_edge(co_edge).child_edge_id;
        if edge.is_valid_for(self.nb_edges()) {
            append_unique(
                &mut self.change_edge_relations_internal(edge).co_edge_ids,
                co_edge,
            );
        }
    }

    /// `DetachWireFromFace(theParentFaceId, theRefId)`: when the wire keeps no other use
    /// on the face, its coedges on that face lose their face-scoped representations.
    pub fn detach_wire_from_face(&mut self, parent_face: FaceId, ref_id: WireRefId) -> bool {
        if !parent_face.is_valid_for(self.nb_faces()) || !ref_id.is_valid_for(self.nb_wire_refs()) {
            return false;
        }
        let wire = self.wire_ref(ref_id).child_wire_id;
        let face_list = &mut self
            .change_face_relations_internal(parent_face)
            .wire_ref_ids;
        if !erase_first(face_list, ref_id) {
            return false;
        }
        if wire.is_valid_for(self.nb_wires()) {
            erase_first(
                &mut self
                    .change_wire_relations_internal(wire)
                    .parent_wire_ref_ids,
                ref_id,
            );
            if !self.wire_has_face_use(wire, parent_face) {
                self.clear_wire_face_context(wire, parent_face);
            }
        }
        true
    }

    /// `DetachFaceFromShell(theParentShellId, theRefId)`: the incoming list of the face is
    /// updated even when the shell did not hold the reference.
    pub fn detach_face_from_shell(&mut self, parent_shell: ShellId, ref_id: FaceRefId) -> bool {
        if !parent_shell.is_valid_for(self.nb_shells()) || !ref_id.is_valid_for(self.nb_face_refs())
        {
            return false;
        }
        let face = self.face_ref(ref_id).child_face_id;
        let erased = erase_first(
            &mut self
                .change_shell_relations_internal(parent_shell)
                .face_ref_ids,
            ref_id,
        );
        if face.is_valid_for(self.nb_faces()) {
            erase_first(
                &mut self
                    .change_face_relations_internal(face)
                    .parent_face_ref_ids,
                ref_id,
            );
        }
        erased
    }

    /// `DetachShellFromSolid(theParentSolidId, theRefId)`.
    pub fn detach_shell_from_solid(&mut self, parent_solid: SolidId, ref_id: ShellRefId) -> bool {
        if !parent_solid.is_valid_for(self.nb_solids())
            || !ref_id.is_valid_for(self.nb_shell_refs())
        {
            return false;
        }
        let shell = self.shell_ref(ref_id).child_shell_id;
        let erased = erase_first(
            &mut self
                .change_solid_relations_internal(parent_solid)
                .shell_ref_ids,
            ref_id,
        );
        if shell.is_valid_for(self.nb_shells()) {
            let incoming = &mut self
                .change_shell_relations_internal(shell)
                .parent_shell_ref_ids;
            erase_first(incoming, ref_id);
        }
        erased
    }

    /// `DetachSolidFromCompSolid(theParentCompSolidId, theRefId)`.
    pub fn detach_solid_from_comp_solid(
        &mut self,
        parent_comp_solid: CompSolidId,
        ref_id: SolidRefId,
    ) -> bool {
        if !parent_comp_solid.is_valid_for(self.nb_comp_solids())
            || !ref_id.is_valid_for(self.nb_solid_refs())
        {
            return false;
        }
        let solid = self.solid_ref(ref_id).child_solid_id;
        let list = &mut self
            .change_comp_solid_relations_internal(parent_comp_solid)
            .solid_ref_ids;
        let erased = erase_first(list, ref_id);
        if solid.is_valid_for(self.nb_solids()) {
            let incoming = &mut self
                .change_solid_relations_internal(solid)
                .parent_solid_ref_ids;
            erase_first(incoming, ref_id);
        }
        erased
    }

    /// `DetachChildFromCompound(theParentCompoundId, theRefId)`: clears the child's
    /// compound-parent bit when its last compound use goes.
    pub fn detach_child_from_compound(
        &mut self,
        parent_compound: CompoundId,
        ref_id: ChildRefId,
    ) -> bool {
        if !parent_compound.is_valid_for(self.nb_compounds())
            || !ref_id.is_valid_for(self.nb_child_refs())
        {
            return false;
        }
        let child = self.child_ref(ref_id).child_node_id;
        let list = &mut self
            .change_compound_relations_internal(parent_compound)
            .child_ref_ids;
        let erased = erase_first(list, ref_id);
        if erased && child.is_valid() {
            self.forget_compound_use(child, ref_id);
        }
        erased
    }

    /// `DetachOccurrenceFromProduct(theParentProductId, theRefId)`: clears the child's
    /// occurrence-parent bit when its last occurrence use goes.
    pub fn detach_occurrence_from_product(
        &mut self,
        parent_product: ProductId,
        ref_id: OccurrenceRefId,
    ) -> bool {
        if !parent_product.is_valid_for(self.nb_products())
            || !ref_id.is_valid_for(self.nb_occurrence_refs())
        {
            return false;
        }
        let occurrence = self.occurrence_ref(ref_id).child_occurrence_id;
        let in_range = occurrence.is_valid_for(self.nb_occurrences());
        let child = if in_range {
            self.occurrence(occurrence).child_node_id
        } else {
            NodeId::default()
        };
        let list = &mut self
            .change_product_relations_internal(parent_product)
            .occurrence_ref_ids;
        let erased = erase_first(list, ref_id);
        if erased && in_range {
            let incoming = &mut self
                .change_occurrence_relations_internal(occurrence)
                .parent_occurrence_ref_ids;
            erase_first(incoming, ref_id);
        }
        if erased && child.is_valid() {
            self.forget_occurrence_use(child, ref_id);
        }
        erased
    }

    /// The sparse-map part of a compound detach or rebind: drops the reference from the
    /// child's incoming list and clears its bit when the list becomes empty.
    pub(crate) fn forget_compound_use(&mut self, child: NodeId, ref_id: ChildRefId) {
        let Some(refs) = self.node_to_compounds.get_mut(&child) else {
            return;
        };
        erase_first(refs, ref_id);
        if refs.is_empty() {
            self.set_has_compound_parent(child, false);
        }
    }

    /// The occurrence twin of [`forget_compound_use`](Self::forget_compound_use).
    pub(crate) fn forget_occurrence_use(&mut self, child: NodeId, ref_id: OccurrenceRefId) {
        let Some(refs) = self.node_to_occurrences.get_mut(&child) else {
            return;
        };
        erase_first(refs, ref_id);
        if refs.is_empty() {
            self.set_has_occurrence_parent(child, false);
        }
    }

    /// `wireHasFaceUse`: an active wire reference of the wire has the face as parent.
    pub(crate) fn wire_has_face_use(&self, wire: WireId, face: FaceId) -> bool {
        if !wire.is_valid_for(self.nb_wires()) || !face.is_valid_for(self.nb_faces()) {
            return false;
        }
        self.wire_relations(wire)
            .parent_wire_ref_ids
            .iter()
            .any(|ref_id| {
                ref_id.is_valid_for(self.nb_wire_refs())
                    && !self.is_removed(*ref_id)
                    && self.wire_ref(*ref_id).parent_face_id == face
            })
    }

    /// `clearWireFaceContext`: the active coedges of the wire on the face lose their
    /// p-curve and polygon uses (marked removed) and their face.
    pub(crate) fn clear_wire_face_context(&mut self, wire: WireId, face: FaceId) {
        if !wire.is_valid_for(self.nb_wires()) || !face.is_valid_for(self.nb_faces()) {
            return;
        }
        let co_edges = self.wire_relations(wire).co_edge_ids.clone();
        for co_edge in co_edges {
            if co_edge.is_valid_for(self.nb_co_edges())
                && !self.is_removed(co_edge)
                && self.co_edge(co_edge).face_id == face
            {
                self.clear_co_edge_face_uses(co_edge);
            }
        }
    }

    fn clear_co_edge_face_uses(&mut self, co_edge: CoEdgeId) {
        let def = self.co_edge(co_edge).clone();
        if def.curve2d_rep_id.is_valid() {
            self.mark_rep_removed(def.curve2d_rep_id.into());
        }
        if def.polygon2d_rep_id.is_valid() {
            self.mark_rep_removed(def.polygon2d_rep_id.into());
        }
        if def.polygon_on_tri_rep_id.is_valid() {
            self.mark_rep_removed(def.polygon_on_tri_rep_id.into());
        }
        let changed = self.change_co_edge(co_edge);
        changed.curve2d_rep_id = CoEdgeCurve2DRepId::default();
        changed.polygon2d_rep_id = CoEdgePolygon2DRepId::default();
        changed.polygon_on_tri_rep_id = CoEdgePolygonOnTriRepId::default();
        changed.face_id = FaceId::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph_inc::ParityOrientation;

    #[test]
    fn detach_undoes_attach() {
        let mut storage = Storage::new();
        let (face, wire, edge) = (
            storage.append_face(),
            storage.append_wire(),
            storage.append_edge(),
        );
        let co_edge = storage.create_co_edge_use(wire, edge, face, ParityOrientation::FORWARD);
        let curve = storage.append_co_edge_curve2d_rep();
        storage.change_co_edge(co_edge).curve2d_rep_id = curve;
        let wire_ref = storage.attach_wire_to_face(face, wire, ParityOrientation::FORWARD);
        assert!(storage.detach_wire_from_face(face, wire_ref));
        assert!(storage.is_removed(curve));
        assert!(!storage.co_edge(co_edge).face_id.is_valid());
        let (first, second) = (storage.append_co_edge(), storage.append_co_edge());
        storage.change_co_edge(first).child_edge_id = edge;
        assert!(storage.replace_co_edge_use_with_pair(wire, co_edge, first, second));
        assert_eq!(
            storage.wire_relations(wire).co_edge_ids,
            vec![first, second]
        );
        assert_eq!(storage.edge_relations(edge).co_edge_ids, vec![first]);
        assert!(storage.detach_co_edge_use(wire, first));
        assert!(!storage.detach_co_edge_use(wire, first));
        let shell = storage.append_shell();
        let face_ref = storage.attach_face_to_shell(shell, face, ParityOrientation::FORWARD);
        assert!(storage.detach_face_from_shell(shell, face_ref));
        assert!(storage.face_relations(face).parent_face_ref_ids.is_empty());
    }
}
