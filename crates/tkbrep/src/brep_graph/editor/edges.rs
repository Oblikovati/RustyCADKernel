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
// Rust translation of EditorView::EdgeOps::Add, Reverse, Mut and the edge setters of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx, BRepGraph_EditorView_Mut.cxx
// and BRepGraph_EditorView_Setters.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EditorView::EdgeOps`: edge creation, reversal and the edge setters. The vertex
//! reference members are in `edge_vertices` and `Split` in `edge_split`.
use super::guards::mutate_node;
use super::helpers::{is_active_node, require_settable};
use crate::brep_graph::mut_guard::MutGuard;
use crate::brep_graph::{BRepGraph, EdgeId, EdgeTag, VertexId, VertexRefId};
use crate::brep_graph_inc::{ParityOrientation, Storage};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::poly::Polygon3D;

/// `EditorView::EdgeOps`.
#[derive(Debug)]
pub struct EdgeOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

impl<'g> EdgeOps<'g> {
    /// `Add(theStartVtx, theEndVtx, theCurve, theFirst, theLast, theTolerance)`: a new
    /// edge whose boundary vertex references are FORWARD (start) and REVERSED (end);
    /// the invalid id when a given vertex is not active. A null curve (a degenerated
    /// edge) gives no 3D curve record.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::{BRepGraph, VertexId};
    /// use rustycad_tkernel::standard::Handle;
    /// use rustycad_tkmath::gp::Point3;
    /// let mut graph = BRepGraph::new();
    /// let start = graph.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
    /// let edge = graph.editor().edges().add(start, VertexId::default(), &Handle::default(), 0.0, 1.0, 1.0e-7);
    /// assert!(edge.is_valid());
    /// ```
    pub fn add(
        &mut self,
        start: VertexId,
        end: VertexId,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> EdgeId {
        let is_usable = |vertex: VertexId| !vertex.is_valid() || is_active_node(self.graph, vertex);
        if !is_usable(start) || !is_usable(end) {
            return EdgeId::default();
        }
        let edge = self.graph.inc_storage_mut().append_edge();
        if start.is_valid() {
            let reference = self.add_boundary_ref(edge, start, ParityOrientation::FORWARD);
            self.graph
                .inc_storage_mut()
                .change_edge(edge)
                .start_vertex_ref_id = reference;
        }
        if end.is_valid() {
            let reference = self.add_boundary_ref(edge, end, ParityOrientation::REVERSED);
            self.graph
                .inc_storage_mut()
                .change_edge(edge)
                .end_vertex_ref_id = reference;
        }
        self.graph.inc_storage_mut().change_edge(edge).tolerance = tolerance;
        self.graph.allocate_uid(edge.into());
        if curve.is_some() {
            attach_new_curve(self.graph.inc_storage_mut(), edge, curve, first, last);
        }
        self.graph.mark_modified(edge.into());
        edge
    }

    // A boundary vertex reference of a new edge, with its UID, and the vertex-to-edge
    // relation (the edge slot is set by the caller).
    fn add_boundary_ref(
        &mut self,
        edge: EdgeId,
        vertex: VertexId,
        orientation: ParityOrientation,
    ) -> VertexRefId {
        let storage = self.graph.inc_storage_mut();
        let reference = storage.append_vertex_ref();
        let entry = storage.change_vertex_ref(reference);
        entry.child_vertex_id = vertex;
        entry.parent_edge_id = edge;
        entry.orientation = orientation;
        self.graph.allocate_ref_uid(reference.into());
        self.graph
            .inc_storage_mut()
            .attach_edge_to_vertex(edge, vertex);
        reference
    }

    /// `Mut(theEdge)`: the scoped guard of an edge definition.
    pub fn mutate(self, edge: EdgeId) -> Result<MutGuard<'g, EdgeTag>, Failure> {
        mutate_node(self.graph, edge)
    }

    /// `Reverse(theEdge)`: swaps the start and end vertex references (the range is
    /// kept); an inactive edge is ignored. An edge out of range is refused by a
    /// debug-only assertion natively and ignored here, as the release build does.
    pub fn reverse(&mut self, edge: EdgeId) {
        if !is_active_node(self.graph, edge) {
            return;
        }
        let def = self.graph.inc_storage_mut().change_edge(edge);
        std::mem::swap(&mut def.start_vertex_ref_id, &mut def.end_vertex_ref_id);
        self.graph.mark_modified(edge.into());
    }

    /// `SetTolerance(theEdge, theTolerance)`.
    pub fn set_tolerance(&mut self, edge: EdgeId, tolerance: f64) -> Result<(), Failure> {
        require_settable(self.graph, edge)?;
        self.graph.inc_storage_mut().change_edge(edge).tolerance = tolerance;
        self.graph.mark_modified(edge.into());
        Ok(())
    }

    /// `SetTolerance(theMut, theTolerance)`.
    pub fn set_tolerance_guarded(guard: &mut MutGuard<'_, EdgeTag>, tolerance: f64) {
        guard.internal().tolerance = tolerance;
    }

    /// `SetParamRange(theEdge, theFirst, theLast)`: the range of the 3D curve record
    /// in range (the edge is marked modified even without one).
    pub fn set_param_range(&mut self, edge: EdgeId, first: f64, last: f64) -> Result<(), Failure> {
        require_settable(self.graph, edge)?;
        set_curve_range(self.graph.inc_storage_mut(), edge, first, last);
        self.graph.mark_modified(edge.into());
        Ok(())
    }

    /// `SetParamRange(theMut, theFirst, theLast)`: `Internal()` flags the guard even
    /// without a curve record.
    pub fn set_param_range_guarded(guard: &mut MutGuard<'_, EdgeTag>, first: f64, last: f64) {
        // `Internal()` flags the guard dirty before the record is looked up.
        guard.mark_dirty();
        let edge = guard.id();
        set_curve_range(guard.graph_mut().inc_storage_mut(), edge, first, last);
    }

    /// `SetCurve(theEdge, theCurve, theFirst, theLast)`: revives and rewrites the
    /// edge's curve record in range, or appends one; a null curve removes the record.
    pub fn set_curve(
        &mut self,
        edge: EdgeId,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        require_settable(self.graph, edge)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.edge(edge).curve3d_rep_id;
        if curve.is_null() {
            storage.mark_rep_removed(rep.into());
        } else if rep.is_valid_for(storage.nb_edge_curves3d()) {
            storage.set_removed(rep, false);
            write_curve(storage, rep, edge, curve, first, last);
        } else {
            attach_new_curve(storage, edge, curve, first, last);
        }
        self.graph.mark_modified(edge.into());
        Ok(())
    }

    /// `ClearCurve(theEdge)`: removes the curve record (the id stays on the edge).
    pub fn clear_curve(&mut self, edge: EdgeId) -> Result<(), Failure> {
        require_settable(self.graph, edge)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.edge(edge).curve3d_rep_id;
        storage.mark_rep_removed(rep.into());
        self.graph.mark_modified(edge.into());
        Ok(())
    }

    /// `SetPersistentPolygon3D(theEdge, thePolygon)`: as [`set_curve`](Self::set_curve)
    /// for the 3D polygon record.
    pub fn set_persistent_polygon3d(
        &mut self,
        edge: EdgeId,
        polygon: &Handle<Polygon3D>,
    ) -> Result<(), Failure> {
        require_settable(self.graph, edge)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.edge(edge).polygon3d_rep_id;
        if polygon.is_null() {
            storage.mark_rep_removed(rep.into());
        } else if rep.is_valid_for(storage.nb_edge_polygons3d()) {
            storage.set_removed(rep, false);
            let record = storage.change_edge_polygon3d_rep(rep);
            record.parent_edge_id = edge;
            record.polygon = polygon.clone();
        } else {
            let rep = storage.append_edge_polygon3d_rep();
            let record = storage.change_edge_polygon3d_rep(rep);
            record.parent_edge_id = edge;
            record.polygon = polygon.clone();
            storage.change_edge(edge).polygon3d_rep_id = rep;
        }
        self.graph.mark_modified(edge.into());
        Ok(())
    }

    /// `ClearPersistentPolygon3D(theEdge)`.
    pub fn clear_persistent_polygon3d(&mut self, edge: EdgeId) -> Result<(), Failure> {
        require_settable(self.graph, edge)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.edge(edge).polygon3d_rep_id;
        storage.mark_rep_removed(rep.into());
        self.graph.mark_modified(edge.into());
        Ok(())
    }
}

// The range update shared by both `SetParamRange` overloads.
fn set_curve_range(storage: &mut Storage, edge: EdgeId, first: f64, last: f64) {
    let rep = storage.edge(edge).curve3d_rep_id;
    if rep.is_valid() && rep.is_valid_for(storage.nb_edge_curves3d()) {
        let record = storage.change_edge_curve3d_rep(rep);
        record.param_first = first;
        record.param_last = last;
    }
}

// A new curve record owned by the edge and bound to it.
fn attach_new_curve(
    storage: &mut Storage,
    edge: EdgeId,
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) {
    let rep = storage.append_edge_curve3d_rep();
    write_curve(storage, rep, edge, curve, first, last);
    storage.change_edge(edge).curve3d_rep_id = rep;
}

fn write_curve(
    storage: &mut Storage,
    rep: crate::brep_graph::EdgeCurve3DRepId,
    edge: EdgeId,
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) {
    let record = storage.change_edge_curve3d_rep(rep);
    record.parent_edge_id = edge;
    record.curve = curve.clone();
    record.param_first = first;
    record.param_last = last;
}
