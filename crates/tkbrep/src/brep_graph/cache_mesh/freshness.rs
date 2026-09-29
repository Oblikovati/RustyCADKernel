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
// Rust translation of the private freshness members of BRepGraph_CacheMesh
// (bindEntry, isCoEdgePolygon2DFresh, isCoEdgePolygonOnTriFresh, isFaceMeshFresh)
// of ModelingData/TKBRep/BRepGraph/BRepGraph_CacheMesh.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Binding the mesh entries and deciding whether they are fresh.
use super::CacheMesh;
use super::entries::{CoEdgeMeshEntry, EdgeMeshEntry, FaceMeshEntry, Slot};
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};

impl CacheMesh {
    /// `bindEntry(theEntry, theFace, theSlot)` (private): bound to the face's
    /// SubtreeGen (a failed bind leaves it unbound) and stamped with the slot.
    pub(super) fn bind_face_entry(
        &self,
        graph: &BRepGraph,
        entry: &mut FaceMeshEntry,
        face: FaceId,
        slot: &Slot,
    ) {
        let _bound = entry.node.bind_subtree_gen(&self.base, graph, face.into());
        entry.stamp = slot.stamp();
    }

    /// `bindEntry(theEntry, theEdge, theSlot)` (private).
    pub(super) fn bind_edge_entry(
        &self,
        graph: &BRepGraph,
        entry: &mut EdgeMeshEntry,
        edge: EdgeId,
        slot: &Slot,
    ) {
        let _bound = entry.node.bind_subtree_gen(&self.base, graph, edge.into());
        entry.stamp = slot.stamp();
    }

    /// `bindEntry(theEntry, theCoEdge, theSlot)` (private): the coedge stamp, then, in
    /// the attached graph, the face topology stamp and the face mesh generation of the
    /// coedge's face (kept as they were when the coedge has no face). A coedge out of
    /// range (natively an unchecked read) binds no face.
    pub(super) fn bind_co_edge_entry(
        &self,
        graph: &BRepGraph,
        entry: &mut CoEdgeMeshEntry,
        co_edge: CoEdgeId,
        slot: &Slot,
    ) {
        let _bound = entry
            .co_edge_stamp
            .bind_subtree_gen(&self.base, graph, co_edge.into());
        if let Some(face) = self.attached_co_edge_face(graph, co_edge) {
            let _face_bound =
                entry
                    .face_topology_stamp
                    .bind_subtree_gen(&self.base, graph, face.into());
            entry.bound_face_id = face;
            entry.face_mesh_generation = slot.face_raw(face).map_or(0, |raw| raw.mesh_generation);
        }
        entry.slot_stamp = slot.stamp();
    }

    // The valid face of a coedge in the attached graph.
    fn attached_co_edge_face(&self, graph: &BRepGraph, co_edge: CoEdgeId) -> Option<FaceId> {
        let graph = self.base.attached_graph(graph)?;
        let storage = graph.inc_storage();
        if !co_edge.is_valid_for(storage.nb_co_edges()) {
            return None;
        }
        let face = storage.co_edge(co_edge).face_id;
        face.is_valid().then_some(face)
    }

    /// The freshness test of `FindFaceMesh` and the dirty collection: present, stamped
    /// with the current slot recipe and generation, and bound to an unchanged
    /// SubtreeGen.
    pub(super) fn is_face_entry_fresh(
        &self,
        graph: &BRepGraph,
        entry: &FaceMeshEntry,
        slot: &Slot,
    ) -> bool {
        entry.is_present() && slot.is_actual(&entry.stamp) && entry.node.is_fresh(&self.base, graph)
    }

    /// The freshness test of `FindEdgeMesh`.
    pub(super) fn is_edge_entry_fresh(
        &self,
        graph: &BRepGraph,
        entry: &EdgeMeshEntry,
        slot: &Slot,
    ) -> bool {
        entry.is_present() && slot.is_actual(&entry.stamp) && entry.node.is_fresh(&self.base, graph)
    }

    /// `isCoEdgePolygon2DFresh(theEntry, theSlot)` (private), also the test of
    /// `findCoEdgeMesh`.
    pub(super) fn is_co_edge_polygon2d_fresh(
        &self,
        graph: &BRepGraph,
        entry: &CoEdgeMeshEntry,
        slot: &Slot,
    ) -> bool {
        slot.is_actual(&entry.slot_stamp) && entry.co_edge_stamp.is_fresh(&self.base, graph)
    }

    /// `isCoEdgePolygonOnTriFresh(theEntry, theSlot)` (private): the 2D test, then, for
    /// a coedge bound to a face, the face topology and the face mesh generation.
    pub(super) fn is_co_edge_polygon_on_tri_fresh(
        &self,
        graph: &BRepGraph,
        entry: &CoEdgeMeshEntry,
        slot: &Slot,
    ) -> bool {
        if !self.is_co_edge_polygon2d_fresh(graph, entry, slot) {
            return false;
        }
        if !entry.face_topology_stamp.is_bound() {
            return true;
        }
        entry.face_topology_stamp.is_fresh(&self.base, graph) && is_face_mesh_fresh(entry, slot)
    }
}

// `isFaceMeshFresh(theEntry, theSlot)` (private): no bound face, or the bound face's
// entry still has the recorded mesh generation.
fn is_face_mesh_fresh(entry: &CoEdgeMeshEntry, slot: &Slot) -> bool {
    let face = entry.bound_face_id;
    if !face.is_valid() {
        return true;
    }
    slot.face_meshes
        .get(face.index as usize)
        .is_some_and(|face_entry| face_entry.mesh_generation == entry.face_mesh_generation)
}
