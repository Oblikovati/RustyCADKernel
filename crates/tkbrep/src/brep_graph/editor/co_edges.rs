// Rust translation of EditorView::CoEdgeOps of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx, BRepGraph_EditorView_Mut.cxx
// and BRepGraph_EditorView_Setters.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EditorView::CoEdgeOps`: coedge creation and the coedge and p-curve setters.
use super::guards::mutate_node;
use super::helpers::require_settable;
use super::helpers::{clear_co_edge_face_scoped_representations, is_active_node};
use crate::brep_graph::mut_guard::MutGuard;
use crate::brep_graph::*;
use crate::brep_graph_inc::{ParityOrientation, Storage, append_unique};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::poly::{Polygon2D, PolygonOnTriangulation};

/// `EditorView::CoEdgeOps`.
#[derive(Debug)]
pub struct CoEdgeOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

impl<'g> CoEdgeOps<'g> {
    /// `Add(theEdge, theOrientation)`: a free coedge (no wire, no face) of an active
    /// edge; the invalid id otherwise.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId};
    /// use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
    /// let mut graph = BRepGraph::new();
    /// let refused = graph.editor().co_edges().add(EdgeId::new(0), ParityOrientation::FORWARD);
    /// assert!(!refused.is_valid());
    /// ```
    pub fn add(&mut self, edge: EdgeId, orientation: ParityOrientation) -> CoEdgeId {
        if !is_active_node(self.graph, edge) {
            return CoEdgeId::default();
        }
        let storage = self.graph.inc_storage_mut();
        let co_edge = storage.append_co_edge();
        let def = storage.change_co_edge(co_edge);
        def.child_edge_id = edge;
        def.orientation = orientation;
        append_unique(
            &mut storage.change_edge_relations_internal(edge).co_edge_ids,
            co_edge,
        );
        self.graph.allocate_uid(co_edge.into());
        self.graph.mark_modified(co_edge.into());
        self.graph.mark_modified(edge.into());
        co_edge
    }

    /// `Add(theEdgeEntity, theFaceEntity, theCurve2d, theFirst, theLast,
    /// theEdgeOrientation)`: a new coedge of the edge on the face with its p-curve
    /// record; the invalid id for an inactive edge or face or a null curve.
    pub fn add_with_pcurve(
        &mut self,
        edge: EdgeId,
        face: FaceId,
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        orientation: ParityOrientation,
    ) -> CoEdgeId {
        if !is_active_node(self.graph, edge) || !is_active_node(self.graph, face) || curve.is_null()
        {
            return CoEdgeId::default();
        }
        let storage = self.graph.inc_storage_mut();
        let co_edge = storage.append_co_edge();
        let def = storage.change_co_edge(co_edge);
        def.child_edge_id = edge;
        def.face_id = face;
        def.orientation = orientation;
        let rep = storage.append_co_edge_curve2d_rep();
        write_pcurve(storage, rep, co_edge, curve, first, last);
        storage.change_co_edge(co_edge).curve2d_rep_id = rep;
        append_unique(
            &mut storage.change_edge_relations_internal(edge).co_edge_ids,
            co_edge,
        );
        self.graph.allocate_uid(co_edge.into());
        self.graph.mark_modified(co_edge.into());
        self.graph.mark_modified(edge.into());
        self.graph.mark_modified(face.into());
        co_edge
    }

    /// `Mut(theCoEdge)`: the scoped guard of a coedge definition.
    pub fn mutate(self, co_edge: CoEdgeId) -> Result<MutGuard<'g, CoEdgeTag>, Failure> {
        mutate_node(self.graph, co_edge)
    }

    /// `SetParamRange(theCoEdge, theFirst, theLast)`: the range of the p-curve record in
    /// range.
    pub fn set_param_range(
        &mut self,
        co_edge: CoEdgeId,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        require_settable(self.graph, co_edge)?;
        set_pcurve_range(self.graph.inc_storage_mut(), co_edge, first, last);
        self.graph.mark_modified(co_edge.into());
        Ok(())
    }

    /// `SetParamRange(theMut, theFirst, theLast)`.
    pub fn set_param_range_guarded(guard: &mut MutGuard<'_, CoEdgeTag>, first: f64, last: f64) {
        // `Internal()` flags the guard dirty before the record is looked up.
        guard.mark_dirty();
        let co_edge = guard.id();
        set_pcurve_range(guard.graph_mut().inc_storage_mut(), co_edge, first, last);
    }

    /// `SetOrientation(theCoEdge, theOrientation)`.
    pub fn set_orientation(
        &mut self,
        co_edge: CoEdgeId,
        orientation: ParityOrientation,
    ) -> Result<(), Failure> {
        require_settable(self.graph, co_edge)?;
        self.graph
            .inc_storage_mut()
            .change_co_edge(co_edge)
            .orientation = orientation;
        self.graph.mark_modified(co_edge.into());
        Ok(())
    }

    /// `SetOrientation(theMut, theOrientation)`.
    pub fn set_orientation_guarded(
        guard: &mut MutGuard<'_, CoEdgeTag>,
        orientation: ParityOrientation,
    ) {
        guard.internal().orientation = orientation;
    }

    /// `SetPCurve(theCoEdge, theCurve2d)`: a null curve clears the p-curve; otherwise
    /// the curve with the range of the active p-curve record of an active coedge, or
    /// the curve's own parameter range.
    pub fn set_pcurve_keep_range(
        &mut self,
        co_edge: CoEdgeId,
        curve: &Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        if curve.is_null() {
            return self.clear_pcurve(co_edge);
        }
        if let Some((first, last)) = active_pcurve_range(self.graph.inc_storage(), co_edge) {
            return self.set_pcurve(co_edge, curve, first, last);
        }
        let (first, last) = (curve.first_parameter(), curve.last_parameter());
        self.set_pcurve(co_edge, curve, first, last)
    }

    /// `SetPCurve(theCoEdge, theCurve2d, theFirst, theLast)`: revives and rewrites the
    /// coedge's p-curve record in range, or appends one; a null curve removes it.
    pub fn set_pcurve(
        &mut self,
        co_edge: CoEdgeId,
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        require_settable(self.graph, co_edge)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.co_edge(co_edge).curve2d_rep_id;
        if curve.is_null() {
            storage.mark_rep_removed(rep.into());
        } else if rep.is_valid_for(storage.nb_co_edge_curves2d()) {
            storage.set_removed(rep, false);
            write_pcurve(storage, rep, co_edge, curve, first, last);
        } else {
            let rep = storage.append_co_edge_curve2d_rep();
            write_pcurve(storage, rep, co_edge, curve, first, last);
            storage.change_co_edge(co_edge).curve2d_rep_id = rep;
        }
        self.graph.mark_modified(co_edge.into());
        Ok(())
    }

    /// `ClearPCurve(theCoEdge)`: removes the p-curve record (the id stays).
    pub fn clear_pcurve(&mut self, co_edge: CoEdgeId) -> Result<(), Failure> {
        require_settable(self.graph, co_edge)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.co_edge(co_edge).curve2d_rep_id;
        storage.mark_rep_removed(rep.into());
        self.graph.mark_modified(co_edge.into());
        Ok(())
    }

    /// `SetPersistentPolygon2D(theCoEdge, thePolygon)`.
    pub fn set_persistent_polygon2d(
        &mut self,
        co_edge: CoEdgeId,
        polygon: &Handle<Polygon2D>,
    ) -> Result<(), Failure> {
        require_settable(self.graph, co_edge)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.co_edge(co_edge).polygon2d_rep_id;
        if polygon.is_null() {
            storage.mark_rep_removed(rep.into());
        } else if rep.is_valid_for(storage.nb_co_edge_polygons2d()) {
            storage.set_removed(rep, false);
            let record = storage.change_co_edge_polygon2d_rep(rep);
            record.parent_co_edge_id = co_edge;
            record.polygon = polygon.clone();
        } else {
            let rep = storage.append_co_edge_polygon2d_rep();
            let record = storage.change_co_edge_polygon2d_rep(rep);
            record.parent_co_edge_id = co_edge;
            record.polygon = polygon.clone();
            storage.change_co_edge(co_edge).polygon2d_rep_id = rep;
        }
        self.graph.mark_modified(co_edge.into());
        Ok(())
    }

    /// `SetPersistentPolygonOnTri(theCoEdge, thePolygon)`.
    pub fn set_persistent_polygon_on_tri(
        &mut self,
        co_edge: CoEdgeId,
        polygon: &Handle<PolygonOnTriangulation>,
    ) -> Result<(), Failure> {
        require_settable(self.graph, co_edge)?;
        let storage = self.graph.inc_storage_mut();
        let rep = storage.co_edge(co_edge).polygon_on_tri_rep_id;
        if polygon.is_null() {
            storage.mark_rep_removed(rep.into());
        } else if rep.is_valid_for(storage.nb_co_edge_polygons_on_tri()) {
            storage.set_removed(rep, false);
            let record = storage.change_co_edge_polygon_on_tri_rep(rep);
            record.parent_co_edge_id = co_edge;
            record.polygon = polygon.clone();
        } else {
            let rep = storage.append_co_edge_polygon_on_tri_rep();
            let record = storage.change_co_edge_polygon_on_tri_rep(rep);
            record.parent_co_edge_id = co_edge;
            record.polygon = polygon.clone();
            storage.change_co_edge(co_edge).polygon_on_tri_rep_id = rep;
        }
        self.graph.mark_modified(co_edge.into());
        Ok(())
    }

    /// `ResetPCurveBinding(theCoEdge)`: unbinds the p-curve record and zeroes its range
    /// (the record stays active).
    pub fn reset_pcurve_binding(&mut self, co_edge: CoEdgeId) -> Result<(), Failure> {
        require_settable(self.graph, co_edge)?;
        reset_binding(self.graph.inc_storage_mut(), co_edge);
        self.graph.mark_modified(co_edge.into());
        Ok(())
    }

    /// `ResetPCurveBinding(theMut)`.
    pub fn reset_pcurve_binding_guarded(guard: &mut MutGuard<'_, CoEdgeTag>) {
        guard.mark_dirty();
        let co_edge = guard.id();
        reset_binding(guard.graph_mut().inc_storage_mut(), co_edge);
    }

    /// `SetChildEdgeId(theCoEdge, theEdge)`: retargets the coedge and rebinds the
    /// edge-to-coedge relation.
    pub fn set_child_edge_id(&mut self, co_edge: CoEdgeId, edge: EdgeId) -> Result<(), Failure> {
        require_settable(self.graph, co_edge)?;
        let storage = self.graph.inc_storage_mut();
        let old = storage.co_edge(co_edge).child_edge_id;
        if old == edge {
            return Ok(());
        }
        storage.change_co_edge(co_edge).child_edge_id = edge;
        storage.rebind_co_edge_edge(co_edge, old, edge);
        self.graph.mark_modified(co_edge.into());
        Ok(())
    }

    /// `SetChildEdgeId(theMut, theEdge)`.
    pub fn set_child_edge_id_guarded(guard: &mut MutGuard<'_, CoEdgeTag>, edge: EdgeId) {
        let old = guard.child_edge_id;
        if old == edge {
            return;
        }
        guard.internal().child_edge_id = edge;
        let co_edge = guard.id();
        guard
            .graph_mut()
            .inc_storage_mut()
            .rebind_co_edge_edge(co_edge, old, edge);
    }

    /// `SetFaceId(theCoEdge, theFace)`: rebinds the owning face and drops the
    /// face-scoped representations.
    pub fn set_face_id(&mut self, co_edge: CoEdgeId, face: FaceId) -> Result<(), Failure> {
        require_settable(self.graph, co_edge)?;
        let storage = self.graph.inc_storage_mut();
        if storage.co_edge(co_edge).face_id == face {
            return Ok(());
        }
        storage.change_co_edge(co_edge).face_id = face;
        clear_co_edge_face_scoped_representations(storage, co_edge);
        self.graph.mark_modified(co_edge.into());
        Ok(())
    }

    /// `SetFaceId(theMut, theFace)`.
    pub fn set_face_id_guarded(guard: &mut MutGuard<'_, CoEdgeTag>, face: FaceId) {
        if guard.face_id == face {
            return;
        }
        guard.internal().face_id = face;
        let co_edge = guard.id();
        clear_co_edge_face_scoped_representations(guard.graph_mut().inc_storage_mut(), co_edge);
    }
}

// The range of the active p-curve record of an active coedge.
fn active_pcurve_range(storage: &Storage, co_edge: CoEdgeId) -> Option<(f64, f64)> {
    if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
        return None;
    }
    let rep = storage.co_edge(co_edge).curve2d_rep_id;
    if !rep.is_valid_for(storage.nb_co_edge_curves2d()) || storage.is_removed(rep) {
        return None;
    }
    let record = storage.co_edge_curve2d_rep(rep);
    Some((record.param_first, record.param_last))
}

fn set_pcurve_range(storage: &mut Storage, co_edge: CoEdgeId, first: f64, last: f64) {
    let rep = storage.co_edge(co_edge).curve2d_rep_id;
    if rep.is_valid() && rep.is_valid_for(storage.nb_co_edge_curves2d()) {
        let record = storage.change_co_edge_curve2d_rep(rep);
        record.param_first = first;
        record.param_last = last;
    }
}

fn reset_binding(storage: &mut Storage, co_edge: CoEdgeId) {
    let old = storage.co_edge(co_edge).curve2d_rep_id;
    storage.change_co_edge(co_edge).curve2d_rep_id = CoEdgeCurve2DRepId::default();
    if old.is_valid() && old.is_valid_for(storage.nb_co_edge_curves2d()) {
        let record = storage.change_co_edge_curve2d_rep(old);
        record.param_first = 0.0;
        record.param_last = 0.0;
    }
}

fn write_pcurve(
    storage: &mut Storage,
    rep: CoEdgeCurve2DRepId,
    co_edge: CoEdgeId,
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) {
    let record = storage.change_co_edge_curve2d_rep(rep);
    record.parent_co_edge_id = co_edge;
    record.curve = curve.clone();
    record.param_first = first;
    record.param_last = last;
}
