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
// Rust translation of EditorView::VertexOps of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx, BRepGraph_EditorView_Mut.cxx
// and BRepGraph_EditorView_Setters.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EditorView::VertexOps`: vertex creation and the vertex setters.
use super::guards::{mutate_node, mutate_ref};
use super::helpers::{node_item, ref_item, require_item_unlocked};
use crate::brep_graph::mut_guard::MutGuard;
use crate::brep_graph::{BRepGraph, VertexId, VertexRefId, VertexRefTag, VertexTag};
use crate::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// `EditorView::VertexOps`.
#[derive(Debug)]
pub struct VertexOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

impl<'g> VertexOps<'g> {
    /// `Add(thePoint, theTolerance)`: a new vertex definition.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// use rustycad_tkmath::gp::Point3;
    /// let mut graph = BRepGraph::new();
    /// let vertex = graph.editor().vertices().add(Point3::new(1.0, 2.0, 3.0), 0.1);
    /// assert_eq!(graph.topo().vertices().definition(vertex).tolerance, 0.1);
    /// ```
    pub fn add(&mut self, point: Point3, tolerance: f64) -> VertexId {
        let vertex = self.graph.inc_storage_mut().append_vertex();
        let def = self.graph.inc_storage_mut().change_vertex(vertex);
        def.point = point;
        def.tolerance = tolerance;
        self.graph.allocate_uid(vertex.into());
        self.graph.mark_modified(vertex.into());
        vertex
    }

    /// `Mut(theVertex)`: the scoped guard of an active, unlocked, unguarded vertex.
    pub fn mutate(self, vertex: VertexId) -> Result<MutGuard<'g, VertexTag>, Failure> {
        mutate_node(self.graph, vertex)
    }

    /// `MutRef(theVertexRef)`: the scoped guard of a vertex reference.
    pub fn mutate_ref(self, reference: VertexRefId) -> Result<MutGuard<'g, VertexRefTag>, Failure> {
        mutate_ref(self.graph, reference)
    }

    /// `SetPoint(theVertex, thePoint)`: immediate notification; a locked vertex raises
    /// Standard_ProgramError.
    pub fn set_point(&mut self, vertex: VertexId, point: Point3) -> Result<(), Failure> {
        require_item_unlocked(self.graph, node_item(vertex))?;
        self.graph.inc_storage_mut().change_vertex(vertex).point = point;
        self.graph.mark_modified(vertex.into());
        Ok(())
    }

    /// `SetPoint(theMut, thePoint)`: inside a guard scope, notified when it closes.
    pub fn set_point_guarded(guard: &mut MutGuard<'_, VertexTag>, point: Point3) {
        guard.internal().point = point;
    }

    /// `SetTolerance(theVertex, theTolerance)`.
    pub fn set_tolerance(&mut self, vertex: VertexId, tolerance: f64) -> Result<(), Failure> {
        require_item_unlocked(self.graph, node_item(vertex))?;
        self.graph.inc_storage_mut().change_vertex(vertex).tolerance = tolerance;
        self.graph.mark_modified(vertex.into());
        Ok(())
    }

    /// `SetTolerance(theMut, theTolerance)`.
    pub fn set_tolerance_guarded(guard: &mut MutGuard<'_, VertexTag>, tolerance: f64) {
        guard.internal().tolerance = tolerance;
    }

    /// `SetRefOrientation(theVertexRef, theOrientation)`.
    pub fn set_ref_orientation(
        &mut self,
        reference: VertexRefId,
        orientation: ParityOrientation,
    ) -> Result<(), Failure> {
        require_item_unlocked(self.graph, ref_item(reference))?;
        self.graph
            .inc_storage_mut()
            .change_vertex_ref(reference)
            .orientation = orientation;
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefOrientation(theMut, theOrientation)`.
    pub fn set_ref_orientation_guarded(
        guard: &mut MutGuard<'_, VertexRefTag>,
        orientation: ParityOrientation,
    ) {
        guard.internal().orientation = orientation;
    }

    /// `SetRefChildVertexId(theVertexRef, theVertex)`: retargets the reference and
    /// rebinds the vertex-to-edge relation of its edge.
    pub fn set_ref_child_vertex_id(
        &mut self,
        reference: VertexRefId,
        vertex: VertexId,
    ) -> Result<(), Failure> {
        require_item_unlocked(self.graph, ref_item(reference))?;
        let old = self
            .graph
            .inc_storage()
            .vertex_ref(reference)
            .child_vertex_id;
        if old == vertex {
            return Ok(());
        }
        self.graph
            .inc_storage_mut()
            .rebind_vertex_ref(reference, old, vertex);
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefChildVertexId(theMut, theVertex)`: the rebind writes the reference
    /// directly, without flagging the guard dirty, as natively.
    pub fn set_ref_child_vertex_id_guarded(
        guard: &mut MutGuard<'_, VertexRefTag>,
        vertex: VertexId,
    ) {
        let old = guard.child_vertex_id;
        if old == vertex {
            return;
        }
        let reference = guard.id();
        guard
            .graph_mut()
            .inc_storage_mut()
            .rebind_vertex_ref(reference, old, vertex);
    }
}
