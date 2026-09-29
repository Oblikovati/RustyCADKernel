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
// Rust translation of EditorView::FaceOps of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx, BRepGraph_EditorView_Mut.cxx
// and BRepGraph_EditorView_Setters.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EditorView::FaceOps`: face creation, wire uses and the face setters.
use super::guards::{mutate_node, mutate_ref};
use super::helpers::{
    is_active_node, mark_co_edge_representations_removed, node_item, require_mutable,
    require_settable,
};
use super::removals::detach_ordered_parent_ref;
use crate::brep_graph::mut_guard::MutGuard;
use crate::brep_graph::*;
use crate::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::poly::Triangulation;

/// `EditorView::FaceOps`.
#[derive(Debug)]
pub struct FaceOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

impl<'g> FaceOps<'g> {
    /// `Add(theSurface, theOuterWire, theInnerWires, theTolerance)`: a face with a
    /// surface record (when the surface is not null) and one wire reference per valid
    /// wire (the outer first); the invalid id when a given wire is not active.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::{BRepGraph, WireId};
    /// use rustycad_tkernel::standard::Handle;
    /// let mut graph = BRepGraph::new();
    /// let face = graph.editor().faces().add(&Handle::default(), WireId::default(), &[], 1.0e-7);
    /// assert!(face.is_valid());
    /// ```
    pub fn add(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        outer: WireId,
        inner: &[WireId],
        tolerance: f64,
    ) -> FaceId {
        let usable = |wire: &WireId| !wire.is_valid() || is_active_node(self.graph, *wire);
        if !usable(&outer) || !inner.iter().all(usable) {
            return FaceId::default();
        }
        let storage = self.graph.inc_storage_mut();
        let face = storage.append_face();
        storage.change_face(face).tolerance = tolerance;
        self.graph.allocate_uid(face.into());
        if surface.is_some() {
            let storage = self.graph.inc_storage_mut();
            let rep = storage.append_face_surface_rep();
            let record = storage.change_face_surface_rep(rep);
            record.parent_face_id = face;
            record.surface = surface.clone();
            storage.change_face(face).surface_rep_id = rep;
        }
        for wire in std::iter::once(&outer)
            .chain(inner)
            .filter(|wire| wire.is_valid())
        {
            let storage = self.graph.inc_storage_mut();
            let reference = storage.attach_wire_to_face(face, *wire, ParityOrientation::FORWARD);
            self.graph.allocate_ref_uid(reference.into());
        }
        self.graph.mark_modified(face.into());
        face
    }

    /// `Append(theFaceEntity, theWireEntity, theOri)`: a wire reference under an active
    /// face; invalid when either is inactive; a locked or guarded face raises
    /// Standard_ProgramError.
    pub fn append(
        &mut self,
        face: FaceId,
        wire: WireId,
        orientation: ParityOrientation,
    ) -> Result<WireRefId, Failure> {
        if !is_active_node(self.graph, face) || !is_active_node(self.graph, wire) {
            return Ok(WireRefId::default());
        }
        require_mutable(
            self.graph,
            node_item(face),
            "BRepGraph::EditorView::Append(): locked face",
            "BRepGraph::EditorView::Append(): guard active on face",
        )?;
        let reference = self
            .graph
            .inc_storage_mut()
            .attach_wire_to_face(face, wire, orientation);
        self.graph.allocate_ref_uid(reference.into());
        self.graph.mark_ref_modified(reference.into());
        Ok(reference)
    }

    /// `RemoveWire(theFaceId, theWireRefId)`: detaches one wire use of an active face,
    /// removes the wire subgraph when it has no other use, and drops the face-scoped
    /// representations of the wire's coedges on this face that lost their face; false
    /// when the reference is not an active use of the face to an active wire (and for
    /// ids out of range, the release behaviour of the native assertions).
    pub fn remove_wire(&mut self, face: FaceId, reference: WireRefId) -> Result<bool, Failure> {
        let storage = self.graph.inc_storage();
        if !face.is_valid_for(storage.nb_faces())
            || !reference.is_valid_for(storage.nb_wire_refs())
            || !is_active_node(self.graph, face)
        {
            return Ok(false);
        }
        let wire = storage.wire_ref(reference).child_wire_id;
        let listed = storage
            .face_relations(face)
            .wire_ref_ids
            .contains(&reference);
        if storage.is_removed(reference)
            || !listed
            || !wire.is_valid()
            || !is_active_node(self.graph, wire)
        {
            return Ok(false);
        }
        let affected = face_co_edges_of_wire(self.graph, wire, face);
        if !detach_ordered_parent_ref(self.graph, reference.into(), listed, wire.into(), true)? {
            return Ok(false);
        }
        for co_edge in affected {
            let storage = self.graph.inc_storage_mut();
            let unbound = co_edge.is_valid_for(storage.nb_co_edges())
                && !storage.is_removed(co_edge)
                && !storage
                    .co_edge(co_edge)
                    .face_id
                    .is_valid_for(storage.nb_faces());
            if unbound {
                mark_co_edge_representations_removed(storage, co_edge);
                self.graph.mark_modified(co_edge.into());
            }
        }
        self.graph.mark_modified(face.into());
        Ok(true)
    }

    /// `Mut(theFace)`: the scoped guard of a face definition.
    pub fn mutate(self, face: FaceId) -> Result<MutGuard<'g, FaceTag>, Failure> {
        mutate_node(self.graph, face)
    }

    /// `MutRef(theFaceRef)`: the scoped guard of a face reference.
    pub fn mutate_ref(self, reference: FaceRefId) -> Result<MutGuard<'g, FaceRefTag>, Failure> {
        mutate_ref(self.graph, reference)
    }

    /// `SetTolerance(theFace, theTolerance)`.
    pub fn set_tolerance(&mut self, face: FaceId, tolerance: f64) -> Result<(), Failure> {
        require_settable(self.graph, face)?;
        self.graph.inc_storage_mut().change_face(face).tolerance = tolerance;
        self.graph.mark_modified(face.into());
        Ok(())
    }

    /// `SetTolerance(theMut, theTolerance)`.
    pub fn set_tolerance_guarded(guard: &mut MutGuard<'_, FaceTag>, tolerance: f64) {
        guard.internal().tolerance = tolerance;
    }

    /// `SetSurface(theFace, theSurface)`: revives and rewrites the face's surface
    /// record in range, or appends one; a null surface removes the record.
    pub fn set_surface(
        &mut self,
        face: FaceId,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        require_settable(self.graph, face)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.face(face).surface_rep_id;
        if surface.is_null() {
            storage.mark_rep_removed(rep.into());
        } else if rep.is_valid_for(storage.nb_face_surfaces()) {
            storage.set_removed(rep, false);
            let record = storage.change_face_surface_rep(rep);
            record.parent_face_id = face;
            record.surface = surface.clone();
        } else {
            let rep = storage.append_face_surface_rep();
            let record = storage.change_face_surface_rep(rep);
            record.parent_face_id = face;
            record.surface = surface.clone();
            storage.change_face(face).surface_rep_id = rep;
        }
        self.graph.mark_modified(face.into());
        Ok(())
    }

    /// `ClearSurface(theFace)`: removes the surface record (the id stays).
    pub fn clear_surface(&mut self, face: FaceId) -> Result<(), Failure> {
        require_settable(self.graph, face)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.face(face).surface_rep_id;
        storage.mark_rep_removed(rep.into());
        self.graph.mark_modified(face.into());
        Ok(())
    }

    /// `SetPersistentTriangulation(theFace, theTriangulation)`.
    pub fn set_persistent_triangulation(
        &mut self,
        face: FaceId,
        triangulation: &Handle<Triangulation>,
    ) -> Result<(), Failure> {
        require_settable(self.graph, face)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.face(face).triangulation_rep_id;
        if triangulation.is_null() {
            storage.mark_rep_removed(rep.into());
        } else if rep.is_valid_for(storage.nb_face_triangulations()) {
            storage.set_removed(rep, false);
            let record = storage.change_face_triangulation_rep(rep);
            record.parent_face_id = face;
            record.triangulation = triangulation.clone();
        } else {
            let rep = storage.append_face_triangulation_rep();
            let record = storage.change_face_triangulation_rep(rep);
            record.parent_face_id = face;
            record.triangulation = triangulation.clone();
            storage.change_face(face).triangulation_rep_id = rep;
        }
        self.graph.mark_modified(face.into());
        Ok(())
    }

    /// `ClearPersistentTriangulation(theFace)`.
    pub fn clear_persistent_triangulation(&mut self, face: FaceId) -> Result<(), Failure> {
        require_settable(self.graph, face)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.face(face).triangulation_rep_id;
        storage.mark_rep_removed(rep.into());
        self.graph.mark_modified(face.into());
        Ok(())
    }

    /// `SetRefOrientation(theFaceRef, theOrientation)`.
    pub fn set_ref_orientation(
        &mut self,
        reference: FaceRefId,
        orientation: ParityOrientation,
    ) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        self.graph
            .inc_storage_mut()
            .change_face_ref(reference)
            .orientation = orientation;
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefOrientation(theMut, theOrientation)`.
    pub fn set_ref_orientation_guarded(
        guard: &mut MutGuard<'_, FaceRefTag>,
        orientation: ParityOrientation,
    ) {
        guard.internal().orientation = orientation;
    }

    /// `SetRefFaceId(theFaceRef, theFace)`: retargets the reference and rebinds the
    /// face-to-shell relations.
    pub fn set_ref_face_id(&mut self, reference: FaceRefId, face: FaceId) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        let storage = self.graph.inc_storage_mut();
        let old = storage.face_ref(reference).child_face_id;
        if old == face {
            return Ok(());
        }
        storage.change_face_ref(reference).child_face_id = face;
        storage.rebind_face_ref(reference, old, face);
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefFaceId(theMut, theFace)`.
    pub fn set_ref_face_id_guarded(guard: &mut MutGuard<'_, FaceRefTag>, face: FaceId) {
        let old = guard.child_face_id;
        if old == face {
            return;
        }
        guard.internal().child_face_id = face;
        let reference = guard.id();
        guard
            .graph_mut()
            .inc_storage_mut()
            .rebind_face_ref(reference, old, face);
    }
}

// The active coedges of the wire on the face (the native affected-edge and face lists
// are collected but never read).
fn face_co_edges_of_wire(graph: &BRepGraph, wire: WireId, face: FaceId) -> Vec<CoEdgeId> {
    let storage = graph.inc_storage();
    let mut affected = Vec::new();
    for co_edge in &storage.wire_relations(wire).co_edge_ids {
        if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(*co_edge) {
            continue;
        }
        if storage.co_edge(*co_edge).face_id == face {
            affected.push(*co_edge);
        }
    }
    affected
}
