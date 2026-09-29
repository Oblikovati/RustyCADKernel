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
// Rust translation of the entry members of BRepGraph_CacheMesh (Has*, Find*, Change*,
// Clear*, BindFresh, BumpFaceMeshGeneration, findFaceEntryRaw, findCoEdgeEntryRaw,
// FindCoEdgePolygon2D, FindCoEdgePolygonOnTri, findCoEdgeMesh) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_CacheMesh.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading, writing and binding the entries of a slot.
use super::entries::{CoEdgeMeshEntry, EdgeMeshEntry, FaceMeshEntry, Slot, valid_index};
use super::{CacheMesh, DEFAULT_DISPLAY_SLOT, SlotId};
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::standard::Failure;

/// `ensureSize(theVec, theIndex)`: appends default entries until `index` exists. The
/// invalid index is refused (natively it asks for 2^32 entries).
pub(super) fn ensure_size<T: Default>(entries: &mut Vec<T>, index: u32) -> Result<&mut T, Failure> {
    if index == u32::MAX {
        return Err(Failure::with_type(
            "Standard_OutOfRange",
            "BRepGraph_CacheMesh: entry of an invalid id (index 4294967295, expected a valid id)",
            "",
        ));
    }
    while entries.len() <= index as usize {
        entries.push(T::default());
    }
    Ok(&mut entries[index as usize])
}

impl CacheMesh {
    /// `FindFaceMesh(theSlot, theFace)`: a copy of the face's entry when it is present
    /// and fresh.
    pub fn find_face_mesh_in(
        &self,
        graph: &BRepGraph,
        slot: SlotId,
        face: FaceId,
    ) -> Option<FaceMeshEntry> {
        let slots = self.slots.read();
        let found = slots.find_slot(slot)?;
        let entry = found
            .face_meshes
            .get(valid_index(face.is_valid(), face.index)?)?;
        self.is_face_entry_fresh(graph, entry, found)
            .then(|| entry.clone())
    }

    /// `FindFaceMesh(theFace)`: the default slot.
    pub fn find_face_mesh(&self, graph: &BRepGraph, face: FaceId) -> Option<FaceMeshEntry> {
        self.find_face_mesh_in(graph, DEFAULT_DISPLAY_SLOT, face)
    }

    /// `HasFaceMesh(theSlot, theFace)`.
    pub fn has_face_mesh_in(&self, graph: &BRepGraph, slot: SlotId, face: FaceId) -> bool {
        self.find_face_mesh_in(graph, slot, face).is_some()
    }

    /// `HasFaceMesh(theFace)`.
    pub fn has_face_mesh(&self, graph: &BRepGraph, face: FaceId) -> bool {
        self.has_face_mesh_in(graph, DEFAULT_DISPLAY_SLOT, face)
    }

    /// `ChangeFaceMesh(theSlot, theFace)`: lends the face's entry (created with its slot
    /// when absent) to `change`. Standard_OutOfRange for the invalid id.
    pub fn change_face_mesh_in<R>(
        &self,
        slot: SlotId,
        face: FaceId,
        change: impl FnOnce(&mut FaceMeshEntry) -> R,
    ) -> Result<R, Failure> {
        let mut slots = self.slots.write();
        let found = slots.change_slot(slot);
        Ok(change(ensure_size(&mut found.face_meshes, face.index)?))
    }

    /// `ChangeFaceMesh(theFace)`.
    pub fn change_face_mesh<R>(
        &self,
        face: FaceId,
        change: impl FnOnce(&mut FaceMeshEntry) -> R,
    ) -> Result<R, Failure> {
        self.change_face_mesh_in(DEFAULT_DISPLAY_SLOT, face, change)
    }

    /// `ClearFaceMesh(theSlot, theFace)`: resets an existing entry.
    pub fn clear_face_mesh_in(&self, slot: SlotId, face: FaceId) {
        let mut slots = self.slots.write();
        let entry = slots
            .find_slot_mut(slot)
            .and_then(|found| found.face_meshes.get_mut(face.index as usize));
        if let Some(entry) = entry.filter(|_| face.is_valid()) {
            entry.reset();
        }
    }

    /// `ClearFaceMesh(theFace)`.
    pub fn clear_face_mesh(&self, face: FaceId) {
        self.clear_face_mesh_in(DEFAULT_DISPLAY_SLOT, face);
    }

    /// `FindEdgeMesh(theSlot, theEdge)`: a copy of the edge's entry when present and
    /// fresh.
    pub fn find_edge_mesh_in(
        &self,
        graph: &BRepGraph,
        slot: SlotId,
        edge: EdgeId,
    ) -> Option<EdgeMeshEntry> {
        let slots = self.slots.read();
        let found = slots.find_slot(slot)?;
        let entry = found
            .edge_meshes
            .get(valid_index(edge.is_valid(), edge.index)?)?;
        self.is_edge_entry_fresh(graph, entry, found)
            .then(|| entry.clone())
    }

    /// `FindEdgeMesh(theEdge)`.
    pub fn find_edge_mesh(&self, graph: &BRepGraph, edge: EdgeId) -> Option<EdgeMeshEntry> {
        self.find_edge_mesh_in(graph, DEFAULT_DISPLAY_SLOT, edge)
    }

    /// `HasEdgeMesh(theSlot, theEdge)`.
    pub fn has_edge_mesh_in(&self, graph: &BRepGraph, slot: SlotId, edge: EdgeId) -> bool {
        self.find_edge_mesh_in(graph, slot, edge).is_some()
    }

    /// `HasEdgeMesh(theEdge)`.
    pub fn has_edge_mesh(&self, graph: &BRepGraph, edge: EdgeId) -> bool {
        self.has_edge_mesh_in(graph, DEFAULT_DISPLAY_SLOT, edge)
    }

    /// `ChangeEdgeMesh(theSlot, theEdge)`: lends the edge's entry to `change`.
    pub fn change_edge_mesh_in<R>(
        &self,
        slot: SlotId,
        edge: EdgeId,
        change: impl FnOnce(&mut EdgeMeshEntry) -> R,
    ) -> Result<R, Failure> {
        let mut slots = self.slots.write();
        let found = slots.change_slot(slot);
        Ok(change(ensure_size(&mut found.edge_meshes, edge.index)?))
    }

    /// `ChangeEdgeMesh(theEdge)`.
    pub fn change_edge_mesh<R>(
        &self,
        edge: EdgeId,
        change: impl FnOnce(&mut EdgeMeshEntry) -> R,
    ) -> Result<R, Failure> {
        self.change_edge_mesh_in(DEFAULT_DISPLAY_SLOT, edge, change)
    }

    /// `ClearEdgeMesh(theSlot, theEdge)`: resets an existing entry.
    pub fn clear_edge_mesh_in(&self, slot: SlotId, edge: EdgeId) {
        let mut slots = self.slots.write();
        let entry = slots
            .find_slot_mut(slot)
            .and_then(|found| found.edge_meshes.get_mut(edge.index as usize));
        if let Some(entry) = entry.filter(|_| edge.is_valid()) {
            entry.reset();
        }
    }

    /// `ClearEdgeMesh(theEdge)`.
    pub fn clear_edge_mesh(&self, edge: EdgeId) {
        self.clear_edge_mesh_in(DEFAULT_DISPLAY_SLOT, edge);
    }

    /// `HasCoEdgeMesh(theSlot, theCoEdge)`: a present entry with a current slot stamp
    /// and fresh coedge topology (`findCoEdgeMesh`).
    pub fn has_co_edge_mesh_in(&self, graph: &BRepGraph, slot: SlotId, co_edge: CoEdgeId) -> bool {
        self.find_co_edge_with(slot, co_edge, |entry, found| {
            self.is_co_edge_polygon2d_fresh(graph, entry, found)
        })
        .is_some()
    }

    /// `HasCoEdgeMesh(theCoEdge)`.
    pub fn has_co_edge_mesh(&self, graph: &BRepGraph, co_edge: CoEdgeId) -> bool {
        self.has_co_edge_mesh_in(graph, DEFAULT_DISPLAY_SLOT, co_edge)
    }

    /// `FindCoEdgePolygon2D(theSlot, theCoEdge)`: the entry when its 2D polygon side is
    /// fresh.
    pub fn find_co_edge_polygon2d_in(
        &self,
        graph: &BRepGraph,
        slot: SlotId,
        co_edge: CoEdgeId,
    ) -> Option<CoEdgeMeshEntry> {
        self.find_co_edge_with(slot, co_edge, |entry, found| {
            self.is_co_edge_polygon2d_fresh(graph, entry, found)
        })
    }

    /// `FindCoEdgePolygon2D(theCoEdge)`.
    pub fn find_co_edge_polygon2d(
        &self,
        graph: &BRepGraph,
        co_edge: CoEdgeId,
    ) -> Option<CoEdgeMeshEntry> {
        self.find_co_edge_polygon2d_in(graph, DEFAULT_DISPLAY_SLOT, co_edge)
    }

    /// `FindCoEdgePolygonOnTri(theSlot, theCoEdge)`: the entry when its polygons on
    /// triangulation are fresh.
    pub fn find_co_edge_polygon_on_tri_in(
        &self,
        graph: &BRepGraph,
        slot: SlotId,
        co_edge: CoEdgeId,
    ) -> Option<CoEdgeMeshEntry> {
        self.find_co_edge_with(slot, co_edge, |entry, found| {
            self.is_co_edge_polygon_on_tri_fresh(graph, entry, found)
        })
    }

    /// `FindCoEdgePolygonOnTri(theCoEdge)`.
    pub fn find_co_edge_polygon_on_tri(
        &self,
        graph: &BRepGraph,
        co_edge: CoEdgeId,
    ) -> Option<CoEdgeMeshEntry> {
        self.find_co_edge_polygon_on_tri_in(graph, DEFAULT_DISPLAY_SLOT, co_edge)
    }

    // The coedge lookups: a present entry of the slot accepted by `fresh`.
    fn find_co_edge_with(
        &self,
        slot: SlotId,
        co_edge: CoEdgeId,
        fresh: impl FnOnce(&CoEdgeMeshEntry, &Slot) -> bool,
    ) -> Option<CoEdgeMeshEntry> {
        let slots = self.slots.read();
        let found = slots.find_slot(slot)?;
        let entry = found
            .co_edge_meshes
            .get(valid_index(co_edge.is_valid(), co_edge.index)?)?;
        (entry.is_present() && fresh(entry, found)).then(|| entry.clone())
    }

    /// `ChangeCoEdgeMesh(theSlot, theCoEdge)`: lends the coedge's entry to `change`.
    pub fn change_co_edge_mesh_in<R>(
        &self,
        slot: SlotId,
        co_edge: CoEdgeId,
        change: impl FnOnce(&mut CoEdgeMeshEntry) -> R,
    ) -> Result<R, Failure> {
        let mut slots = self.slots.write();
        let found = slots.change_slot(slot);
        Ok(change(ensure_size(
            &mut found.co_edge_meshes,
            co_edge.index,
        )?))
    }

    /// `ChangeCoEdgeMesh(theCoEdge)`.
    pub fn change_co_edge_mesh<R>(
        &self,
        co_edge: CoEdgeId,
        change: impl FnOnce(&mut CoEdgeMeshEntry) -> R,
    ) -> Result<R, Failure> {
        self.change_co_edge_mesh_in(DEFAULT_DISPLAY_SLOT, co_edge, change)
    }

    /// `ClearCoEdgeMesh(theSlot, theCoEdge)`: resets an existing entry.
    pub fn clear_co_edge_mesh_in(&self, slot: SlotId, co_edge: CoEdgeId) {
        let mut slots = self.slots.write();
        let entry = slots
            .find_slot_mut(slot)
            .and_then(|found| found.co_edge_meshes.get_mut(co_edge.index as usize));
        if let Some(entry) = entry.filter(|_| co_edge.is_valid()) {
            entry.reset();
        }
    }

    /// `ClearCoEdgeMesh(theCoEdge)`.
    pub fn clear_co_edge_mesh(&self, co_edge: CoEdgeId) {
        self.clear_co_edge_mesh_in(DEFAULT_DISPLAY_SLOT, co_edge);
    }

    /// `findFaceEntryRaw(theSlot, theFace)`: a copy of the face entry with a
    /// triangulation, fresh or not.
    pub fn find_face_entry_raw(&self, slot: SlotId, face: FaceId) -> Option<FaceMeshEntry> {
        let slots = self.slots.read();
        slots.find_slot(slot)?.face_raw(face).cloned()
    }

    /// `findCoEdgeEntryRaw(theSlot, theCoEdge)`: a copy of the coedge entry with a
    /// representation, fresh or not.
    pub fn find_co_edge_entry_raw(
        &self,
        slot: SlotId,
        co_edge: CoEdgeId,
    ) -> Option<CoEdgeMeshEntry> {
        let slots = self.slots.read();
        slots.find_slot(slot)?.co_edge_raw(co_edge).cloned()
    }

    /// `BindFresh(theEntry, theFace)`: binds a written entry against the default slot
    /// (nothing when that slot is missing). Not to be called inside a `change_*`
    /// closure: bind the entry the closure returns, then store it back.
    pub fn bind_fresh_face(&self, graph: &BRepGraph, entry: &mut FaceMeshEntry, face: FaceId) {
        let slots = self.slots.read();
        if let Some(found) = slots.find_slot(DEFAULT_DISPLAY_SLOT) {
            self.bind_face_entry(graph, entry, face, found);
        }
    }

    /// `BindFresh(theEntry, theCoEdge)`.
    pub fn bind_fresh_co_edge(
        &self,
        graph: &BRepGraph,
        entry: &mut CoEdgeMeshEntry,
        co_edge: CoEdgeId,
    ) {
        let slots = self.slots.read();
        if let Some(found) = slots.find_slot(DEFAULT_DISPLAY_SLOT) {
            self.bind_co_edge_entry(graph, entry, co_edge, found);
        }
    }

    /// `BindFresh(theEntry, theEdge)`.
    pub fn bind_fresh_edge(&self, graph: &BRepGraph, entry: &mut EdgeMeshEntry, edge: EdgeId) {
        let slots = self.slots.read();
        if let Some(found) = slots.find_slot(DEFAULT_DISPLAY_SLOT) {
            self.bind_edge_entry(graph, entry, edge, found);
        }
    }

    /// `BumpFaceMeshGeneration(theFace, theSlot)`: the only mutator of the face mesh
    /// generation (the entry is created when absent).
    pub fn bump_face_mesh_generation(&self, face: FaceId, slot: SlotId) -> Result<(), Failure> {
        self.change_face_mesh_in(slot, face, |entry| {
            entry.mesh_generation = entry.mesh_generation.wrapping_add(1);
        })
    }
}
