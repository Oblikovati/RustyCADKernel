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
// Rust translation of the `ChangeXxxMesh(theId)` + `BindFresh(anEntry, theId)` sequences
// of BRepGraph::MeshView::EditorView (ModelingData/TKBRep/BRepGraph/
// BRepGraph_MeshView.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Writing a default-slot entry and binding it fresh under one lock.
//!
//! Natively the mesh editor changes the entry through the reference `Change*Mesh`
//! returns, then binds that same entry with `BindFresh`. Here both steps run under the
//! cache lock: the entry is changed, bound against the default slot, and stored back.
use super::access::ensure_size;
use super::entries::{CoEdgeMeshEntry, EdgeMeshEntry, FaceMeshEntry};
use super::{CacheMesh, DEFAULT_DISPLAY_SLOT};
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::standard::Failure;

impl CacheMesh {
    /// `ChangeFaceMesh(theFace)`, `change`, then `BindFresh(anEntry, theFace)`.
    pub(crate) fn write_face_fresh(
        &self,
        graph: &BRepGraph,
        face: FaceId,
        change: impl FnOnce(&mut FaceMeshEntry),
    ) -> Result<(), Failure> {
        let mut slots = self.slots.write();
        let slot = slots.change_slot(DEFAULT_DISPLAY_SLOT);
        let mut entry = ensure_size(&mut slot.face_meshes, face.index)?.clone();
        change(&mut entry);
        self.bind_face_entry(graph, &mut entry, face, slot);
        slot.face_meshes[face.index as usize] = entry;
        Ok(())
    }

    /// `ChangeEdgeMesh(theEdge)`, `change`, then `BindFresh(anEntry, theEdge)`.
    pub(crate) fn write_edge_fresh(
        &self,
        graph: &BRepGraph,
        edge: EdgeId,
        change: impl FnOnce(&mut EdgeMeshEntry),
    ) -> Result<(), Failure> {
        let mut slots = self.slots.write();
        let slot = slots.change_slot(DEFAULT_DISPLAY_SLOT);
        let mut entry = ensure_size(&mut slot.edge_meshes, edge.index)?.clone();
        change(&mut entry);
        self.bind_edge_entry(graph, &mut entry, edge, slot);
        slot.edge_meshes[edge.index as usize] = entry;
        Ok(())
    }

    /// `ChangeCoEdgeMesh(theCoEdge)`, `change`, then `BindFresh(anEntry, theCoEdge)`.
    pub(crate) fn write_co_edge_fresh(
        &self,
        graph: &BRepGraph,
        co_edge: CoEdgeId,
        change: impl FnOnce(&mut CoEdgeMeshEntry),
    ) -> Result<(), Failure> {
        let mut slots = self.slots.write();
        let slot = slots.change_slot(DEFAULT_DISPLAY_SLOT);
        let mut entry = ensure_size(&mut slot.co_edge_meshes, co_edge.index)?.clone();
        change(&mut entry);
        self.bind_co_edge_entry(graph, &mut entry, co_edge, slot);
        slot.co_edge_meshes[co_edge.index as usize] = entry;
        Ok(())
    }
}
