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
// Rust translation of the entry types of BRepGraph_CacheMesh (EntryStamp,
// FaceMeshEntry, CoEdgeMeshEntry, EdgeMeshEntry, DirtySet, SlotState and the
// private Slot) of ModelingData/TKBRep/BRepGraph/BRepGraph_CacheMesh.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cached mesh entries of one slot and the slot itself.
use super::{CacheMeshDriver, DEFAULT_DISPLAY_SLOT, SlotId};
use crate::brep_graph::cache::NodeEntry;
use crate::brep_graph::{EdgeId, FaceId};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};

/// `BRepGraph_CacheMesh::EntryStamp`: the slot recipe and slot generation an entry was
/// written under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EntryStamp {
    /// `RecipeHash`.
    pub recipe_hash: u64,
    /// `SlotGeneration`.
    pub slot_generation: u32,
}

impl EntryStamp {
    /// `Reset()`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// `BRepGraph_CacheMesh::FaceMeshEntry`: the cached triangulation of a face, bound to
/// the face's SubtreeGen, with the mesh generation its coedge polygons track.
///
/// ```
/// use rustycad_tkbrep::brep_graph::cache_mesh::FaceMeshEntry;
/// let mut entry = FaceMeshEntry::default();
/// assert!(!entry.is_present());
/// entry.mesh_generation = 3;
/// entry.clear_representation();
/// assert_eq!(entry.mesh_generation, 3);
/// ```
#[derive(Clone, Default)]
pub struct FaceMeshEntry {
    /// The `NodeEntry` base.
    pub node: NodeEntry,
    /// `Triangulation`.
    pub triangulation: Handle<Triangulation>,
    /// `Stamp`.
    pub stamp: EntryStamp,
    /// `MeshGeneration`: bumped only by `BumpFaceMeshGeneration`.
    pub mesh_generation: u32,
}

impl FaceMeshEntry {
    /// `IsPresent()`: a triangulation is stored.
    pub fn is_present(&self) -> bool {
        self.triangulation.is_some()
    }

    /// `ClearRepresentation()`: drops the triangulation, keeping the mesh generation.
    pub fn clear_representation(&mut self) {
        self.triangulation = Handle::default();
    }

    /// `Reset()`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// `BRepGraph_CacheMesh::CoEdgeMeshEntry`: the cached polygon-on-surface and
/// polygons-on-triangulation of a coedge, with two freshness dimensions (the coedge
/// topology, and the face topology plus the face mesh content).
#[derive(Clone, Default)]
pub struct CoEdgeMeshEntry {
    /// `CoEdgeStamp`: coedge topology freshness (for the 2D polygon).
    pub co_edge_stamp: NodeEntry,
    /// `FaceTopologyStamp`: face topology freshness (for the polygons on triangulation).
    pub face_topology_stamp: NodeEntry,
    /// `FaceMeshGeneration`: the face mesh generation the polygons were written for.
    pub face_mesh_generation: u32,
    /// `BoundFaceId`: the face whose mesh generation is tracked.
    pub bound_face_id: FaceId,
    /// `SlotStamp`.
    pub slot_stamp: EntryStamp,
    /// `Polygon2D`.
    pub polygon2d: Handle<Polygon2D>,
    /// `PolygonsOnTri`.
    pub polygons_on_tri: Vec<Handle<PolygonOnTriangulation>>,
}

impl CoEdgeMeshEntry {
    /// `IsPresent()`: a 2D polygon or a polygon on triangulation is stored.
    pub fn is_present(&self) -> bool {
        self.polygon2d.is_some() || !self.polygons_on_tri.is_empty()
    }

    /// `Reset()`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// `BRepGraph_CacheMesh::EdgeMeshEntry`: the cached 3D polygon of an edge, bound to the
/// edge's SubtreeGen.
#[derive(Clone, Default)]
pub struct EdgeMeshEntry {
    /// The `NodeEntry` base.
    pub node: NodeEntry,
    /// `Polygon3D`.
    pub polygon3d: Handle<Polygon3D>,
    /// `Stamp`.
    pub stamp: EntryStamp,
}

impl EdgeMeshEntry {
    /// `IsPresent()`.
    pub fn is_present(&self) -> bool {
        self.polygon3d.is_some()
    }

    /// `Reset()`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// `BRepGraph_CacheMesh::DirtySet`: the stale faces and free edges a driver refills.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirtySet {
    /// `Faces`.
    pub faces: Vec<FaceId>,
    /// `FreeEdges`: edges used by no face.
    pub free_edges: Vec<EdgeId>,
}

impl DirtySet {
    /// `IsEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty() && self.free_edges.is_empty()
    }

    /// `Clear()`.
    pub fn clear(&mut self) {
        self.faces.clear();
        self.free_edges.clear();
    }
}

/// `BRepGraph_CacheMesh::SlotState`: the runtime state of a slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotState {
    /// `Slot`.
    pub slot: SlotId,
    /// `RecipeHash`.
    pub recipe_hash: u64,
    /// `Generation`.
    pub generation: u32,
    /// `HasDriver`.
    pub has_driver: bool,
}

impl Default for SlotState {
    fn default() -> Self {
        Self {
            slot: DEFAULT_DISPLAY_SLOT,
            recipe_hash: 0,
            generation: 1,
            has_driver: false,
        }
    }
}

/// `BRepGraph_CacheMesh::Slot` (private): a driver, its recipe, the slot generation
/// and the entries per face, coedge and edge index. The native incremental allocator
/// has no Rust counterpart.
#[derive(Clone, Default)]
pub(super) struct Slot {
    pub(super) id: SlotId,
    pub(super) driver: Handle<dyn CacheMeshDriver>,
    pub(super) recipe_hash: u64,
    pub(super) generation: u32,
    pub(super) face_meshes: Vec<FaceMeshEntry>,
    pub(super) co_edge_meshes: Vec<CoEdgeMeshEntry>,
    pub(super) edge_meshes: Vec<EdgeMeshEntry>,
}

impl Slot {
    /// `Slot()`: generation 1, no driver.
    pub(super) fn new(id: SlotId) -> Self {
        Self {
            id,
            generation: 1,
            ..Self::default()
        }
    }

    /// `Clear()`: drops every entry and starts a new slot generation.
    pub(super) fn clear(&mut self) {
        self.face_meshes = Vec::new();
        self.co_edge_meshes = Vec::new();
        self.edge_meshes = Vec::new();
        self.generation = self.generation.wrapping_add(1);
    }

    /// `isSlotActual(theSlot, theStamp)`: the stamp matches the slot's recipe and
    /// generation.
    pub(super) fn is_actual(&self, stamp: &EntryStamp) -> bool {
        stamp.recipe_hash == self.recipe_hash && stamp.slot_generation == self.generation
    }

    /// The stamp an entry bound now receives.
    pub(super) fn stamp(&self) -> EntryStamp {
        EntryStamp {
            recipe_hash: self.recipe_hash,
            slot_generation: self.generation,
        }
    }

    /// `findFaceEntryRaw`: the face entry with a triangulation (no freshness check).
    pub(super) fn face_raw(&self, face: FaceId) -> Option<&FaceMeshEntry> {
        let entry = self
            .face_meshes
            .get(valid_index(face.is_valid(), face.index)?)?;
        entry.is_present().then_some(entry)
    }

    /// `findCoEdgeEntryRaw`: the coedge entry with a representation.
    pub(super) fn co_edge_raw(
        &self,
        co_edge: crate::brep_graph::CoEdgeId,
    ) -> Option<&CoEdgeMeshEntry> {
        let index = valid_index(co_edge.is_valid(), co_edge.index)?;
        let entry = self.co_edge_meshes.get(index)?;
        entry.is_present().then_some(entry)
    }
}

/// The entry index of an id valid in an entry array (`IsValidIn`), else `None`.
pub(super) fn valid_index(valid: bool, index: u32) -> Option<usize> {
    valid.then_some(index as usize)
}
