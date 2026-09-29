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
// Rust translation of EditorView::EdgeOps::Split, initSubEdgeEntity and
// initSubCoEdgeEntity of ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EdgeOps::Split`: one edge becomes two at a vertex, with every coedge and wire use.
use super::edges::EdgeOps;
use super::helpers::{mark_co_edge_representations_removed, node_item, require_mutable};
use crate::brep_graph::*;
use crate::brep_graph_inc::{ParityOrientation, Storage};
use rustycad_tkernel::standard::Failure;
use std::collections::HashMap;

/// The data of the original edge that `Split` copies before it appends.
struct SplitSource {
    edge: EdgeId,
    curve: EdgeCurve3DRepId,
    polygon: EdgePolygon3DRepId,
    tolerance: f64,
    start_ref: VertexRefId,
    end_ref: VertexRefId,
    first: f64,
    last: f64,
    split: f64,
}

impl EdgeOps<'_> {
    /// `Split(theEdgeEntity, theSplitVertex, theSplitParam, theSubA, theSubB)`: two new
    /// edges (start to split vertex on `[first, split]`, split vertex to end on
    /// `[split, last]`); each coedge of the edge becomes two coedges whose p-curve
    /// ranges are cut at the same ratio, each wire use of the edge becomes the pair,
    /// and the edge, its coedges and its boundary references are removed. Returns
    /// `(SubA, SubB)`; both are invalid when a debug-only assertion of the native code
    /// refuses the input (edge or vertex out of range or removed, parameter outside
    /// the open range), as the release build returns there. A locked or guarded edge
    /// raises Standard_ProgramError.
    pub fn split(
        &mut self,
        edge: EdgeId,
        split_vertex: VertexId,
        split_param: f64,
    ) -> Result<(EdgeId, EdgeId), Failure> {
        let invalid = (EdgeId::default(), EdgeId::default());
        require_mutable(
            self.graph,
            node_item(edge),
            "BRepGraph::EditorView::Split(): owned edge",
            "BRepGraph::EditorView::Split(): guard active on edge",
        )?;
        let Some(source) = split_source(self.graph.inc_storage(), edge, split_vertex, split_param)
        else {
            return Ok(invalid);
        };
        let wires: Vec<WireId> = self.graph.topo().edges().wires_of(edge).collect();
        let (sub_a, sub_b) = self.append_sub_edges(&source, split_vertex);
        self.split_co_edges(&source, sub_a, sub_b, &wires);
        retire_original(self.graph.inc_storage_mut(), &source);
        for wire in &wires {
            self.graph.mark_modified((*wire).into());
        }
        let (start_vertex, end_vertex) = boundary_vertices(self.graph.inc_storage(), &source);
        let storage = self.graph.inc_storage_mut();
        storage.rebind_vertex_edge(
            start_vertex,
            VertexId::default(),
            edge,
            VertexRefId::default(),
        );
        storage.rebind_vertex_edge(
            end_vertex,
            VertexId::default(),
            edge,
            VertexRefId::default(),
        );
        self.graph.mark_modified(edge.into());
        self.graph.mark_modified(sub_a.into());
        self.graph.mark_modified(sub_b.into());
        Ok((sub_a, sub_b))
    }

    // The two sub-edge slots, their four boundary references (in native order), their
    // copied representations and vertex relations, then their UIDs.
    fn append_sub_edges(
        &mut self,
        source: &SplitSource,
        split_vertex: VertexId,
    ) -> (EdgeId, EdgeId) {
        let sub_a = self.graph.inc_storage_mut().append_edge();
        let sub_b = self.graph.inc_storage_mut().append_edge();
        let a_start = self.copy_boundary_ref(source.start_ref, sub_a);
        let a_end = self.new_split_ref(split_vertex, sub_a, ParityOrientation::REVERSED);
        let b_start = self.new_split_ref(split_vertex, sub_b, ParityOrientation::FORWARD);
        let b_end = self.copy_boundary_ref(source.end_ref, sub_b);
        let storage = self.graph.inc_storage_mut();
        init_sub_edge(
            storage,
            source,
            sub_a,
            (a_start, a_end),
            (source.first, source.split),
        );
        init_sub_edge(
            storage,
            source,
            sub_b,
            (b_start, b_end),
            (source.split, source.last),
        );
        self.graph.allocate_uid(sub_a.into());
        self.graph.allocate_uid(sub_b.into());
        (sub_a, sub_b)
    }

    // The sub-edge copy of an original boundary reference (child and orientation).
    fn copy_boundary_ref(&mut self, original: VertexRefId, sub_edge: EdgeId) -> VertexRefId {
        if !original.is_valid() {
            return VertexRefId::default();
        }
        let entry = self.graph.inc_storage().vertex_ref(original).clone();
        self.new_split_ref(entry.child_vertex_id, sub_edge, entry.orientation)
    }

    // A new boundary reference of a sub-edge (the split vertex is valid: `split_source`
    // checked it; a copied reference keeps the original child, valid or not).
    fn new_split_ref(
        &mut self,
        vertex: VertexId,
        sub_edge: EdgeId,
        orientation: ParityOrientation,
    ) -> VertexRefId {
        let storage = self.graph.inc_storage_mut();
        let reference = storage.append_vertex_ref();
        let entry = storage.change_vertex_ref(reference);
        entry.child_vertex_id = vertex;
        entry.parent_edge_id = sub_edge;
        entry.orientation = orientation;
        self.graph.allocate_ref_uid(reference.into());
        reference
    }

    // Steps 1 to 3 of the coedge rebuild: two coedges per original coedge, then the
    // wire uses replaced by the pairs and each wire's order canonicalised.
    fn split_co_edges(
        &mut self,
        source: &SplitSource,
        sub_a: EdgeId,
        sub_b: EdgeId,
        wires: &[WireId],
    ) {
        let storage = self.graph.inc_storage();
        let originals: Vec<CoEdgeId> = if source.edge.is_valid_for(storage.nb_edges()) {
            storage.edge_relations(source.edge).co_edge_ids.clone()
        } else {
            Vec::new()
        };
        let ratio = (source.split - source.first) / (source.last - source.first);
        let mut pairs: HashMap<CoEdgeId, (CoEdgeId, CoEdgeId)> = HashMap::new();
        for original in &originals {
            let old = self.graph.inc_storage().co_edge(*original).clone();
            let first = self.append_sub_co_edge(&old, sub_a, ratio, true);
            let second = self.append_sub_co_edge(&old, sub_b, ratio, false);
            pairs.insert(*original, (first, second));
        }
        for wire in wires {
            replace_wire_uses(self.graph.inc_storage_mut(), *wire, &pairs);
        }
        let storage = self.graph.inc_storage_mut();
        for original in &originals {
            mark_co_edge_representations_removed(storage, *original);
            storage.mark_removed((*original).into());
        }
    }

    // One sub-coedge with its copied representations, then its UID.
    fn append_sub_co_edge(
        &mut self,
        old: &crate::brep_graph_inc::CoEdgeDef,
        sub_edge: EdgeId,
        ratio: f64,
        is_first_part: bool,
    ) -> CoEdgeId {
        let storage = self.graph.inc_storage_mut();
        let co_edge = storage.append_co_edge();
        let curve = copy_co_edge_curve(storage, old.curve2d_rep_id, co_edge, ratio, is_first_part);
        let polygon = copy_co_edge_polygon2d(storage, old.polygon2d_rep_id, co_edge);
        let on_tri = copy_co_edge_polygon_on_tri(storage, old.polygon_on_tri_rep_id, co_edge);
        let def = storage.change_co_edge(co_edge);
        def.child_edge_id = sub_edge;
        def.face_id = old.face_id;
        def.orientation = old.orientation;
        def.curve2d_rep_id = curve;
        def.polygon2d_rep_id = polygon;
        def.polygon_on_tri_rep_id = on_tri;
        self.graph.allocate_uid(co_edge.into());
        co_edge
    }
}

// The native assertions of `Split` (debug-only, so the release build returns there)
// and the copy of the original edge.
fn split_source(
    storage: &Storage,
    edge: EdgeId,
    split_vertex: VertexId,
    split_param: f64,
) -> Option<SplitSource> {
    if !edge.is_valid_for(storage.nb_edges())
        || !split_vertex.is_valid_for(storage.nb_vertices())
        || storage.is_removed(split_vertex)
        || storage.is_removed(edge)
    {
        return None;
    }
    let def = storage.edge(edge);
    let (mut first, mut last) = (0.0, 0.0);
    if def.curve3d_rep_id.is_valid_for(storage.nb_edge_curves3d())
        && !storage.is_removed(def.curve3d_rep_id)
    {
        let record = storage.edge_curve3d_rep(def.curve3d_rep_id);
        (first, last) = (record.param_first, record.param_last);
    }
    if !(first < split_param && split_param < last) {
        return None;
    }
    Some(SplitSource {
        edge,
        curve: def.curve3d_rep_id,
        polygon: def.polygon3d_rep_id,
        tolerance: def.tolerance,
        start_ref: def.start_vertex_ref_id,
        end_ref: def.end_vertex_ref_id,
        first,
        last,
        split: split_param,
    })
}

// `initSubEdgeEntity` plus the copied curve and polygon records and the vertex
// relations of the sub-edge's boundary references.
fn init_sub_edge(
    storage: &mut Storage,
    source: &SplitSource,
    sub_edge: EdgeId,
    (start, end): (VertexRefId, VertexRefId),
    (first, last): (f64, f64),
) {
    let curve = copy_edge_curve(storage, source.curve, sub_edge, first, last);
    let polygon = copy_edge_polygon(storage, source.polygon, sub_edge);
    let def = storage.change_edge(sub_edge);
    def.curve3d_rep_id = curve;
    def.tolerance = source.tolerance;
    def.start_vertex_ref_id = start;
    def.end_vertex_ref_id = end;
    def.polygon3d_rep_id = polygon;
    for reference in [start, end] {
        if reference.is_valid_for(storage.nb_vertex_refs()) {
            let vertex = storage.vertex_ref(reference).child_vertex_id;
            storage.attach_edge_to_vertex(sub_edge, vertex);
        }
    }
}

fn copy_edge_curve(
    storage: &mut Storage,
    original: EdgeCurve3DRepId,
    sub_edge: EdgeId,
    first: f64,
    last: f64,
) -> EdgeCurve3DRepId {
    if !original.is_valid_for(storage.nb_edge_curves3d()) || storage.is_removed(original) {
        return EdgeCurve3DRepId::default();
    }
    let curve = storage.edge_curve3d_rep(original).curve.clone();
    let rep = storage.append_edge_curve3d_rep();
    let record = storage.change_edge_curve3d_rep(rep);
    record.parent_edge_id = sub_edge;
    record.curve = curve;
    record.param_first = first;
    record.param_last = last;
    rep
}

fn copy_edge_polygon(
    storage: &mut Storage,
    original: EdgePolygon3DRepId,
    sub_edge: EdgeId,
) -> EdgePolygon3DRepId {
    if !original.is_valid_for(storage.nb_edge_polygons3d()) || storage.is_removed(original) {
        return EdgePolygon3DRepId::default();
    }
    let polygon = storage.edge_polygon3d_rep(original).polygon.clone();
    let rep = storage.append_edge_polygon3d_rep();
    let record = storage.change_edge_polygon3d_rep(rep);
    record.parent_edge_id = sub_edge;
    record.polygon = polygon;
    rep
}

// The p-curve record of a sub-coedge: the original range cut at the parameter
// `first + (last - first) * ratio`, the first or the second part.
fn copy_co_edge_curve(
    storage: &mut Storage,
    original: CoEdgeCurve2DRepId,
    co_edge: CoEdgeId,
    ratio: f64,
    is_first_part: bool,
) -> CoEdgeCurve2DRepId {
    if !original.is_valid_for(storage.nb_co_edge_curves2d()) || storage.is_removed(original) {
        return CoEdgeCurve2DRepId::default();
    }
    let source = storage.co_edge_curve2d_rep(original).clone();
    let split = source.param_first + (source.param_last - source.param_first) * ratio;
    let rep = storage.append_co_edge_curve2d_rep();
    let record = storage.change_co_edge_curve2d_rep(rep);
    record.parent_co_edge_id = co_edge;
    record.curve = source.curve;
    record.param_first = if is_first_part {
        source.param_first
    } else {
        split
    };
    record.param_last = if is_first_part {
        split
    } else {
        source.param_last
    };
    rep
}

fn copy_co_edge_polygon2d(
    storage: &mut Storage,
    original: CoEdgePolygon2DRepId,
    co_edge: CoEdgeId,
) -> CoEdgePolygon2DRepId {
    if !original.is_valid_for(storage.nb_co_edge_polygons2d()) || storage.is_removed(original) {
        return CoEdgePolygon2DRepId::default();
    }
    let polygon = storage.co_edge_polygon2d_rep(original).polygon.clone();
    let rep = storage.append_co_edge_polygon2d_rep();
    let record = storage.change_co_edge_polygon2d_rep(rep);
    record.parent_co_edge_id = co_edge;
    record.polygon = polygon;
    rep
}

fn copy_co_edge_polygon_on_tri(
    storage: &mut Storage,
    original: CoEdgePolygonOnTriRepId,
    co_edge: CoEdgeId,
) -> CoEdgePolygonOnTriRepId {
    if !original.is_valid_for(storage.nb_co_edge_polygons_on_tri()) || storage.is_removed(original)
    {
        return CoEdgePolygonOnTriRepId::default();
    }
    let polygon = storage.co_edge_polygon_on_tri_rep(original).polygon.clone();
    let rep = storage.append_co_edge_polygon_on_tri_rep();
    let record = storage.change_co_edge_polygon_on_tri_rep(rep);
    record.parent_co_edge_id = co_edge;
    record.polygon = polygon;
    rep
}

// Step 3 for one wire: each active original use replaced by its pair (over a snapshot
// of the uses), then the canonical order.
fn replace_wire_uses(
    storage: &mut Storage,
    wire: WireId,
    pairs: &HashMap<CoEdgeId, (CoEdgeId, CoEdgeId)>,
) {
    let snapshot = storage.wire_relations(wire).co_edge_ids.clone();
    for old in snapshot {
        if storage.is_removed(old) {
            continue;
        }
        if let Some((first, second)) = pairs.get(&old) {
            storage.replace_co_edge_use_with_pair(wire, old, *first, *second);
        }
    }
    storage.canonicalize_wire_co_edge_order(wire);
}

// Step 5 and the end of step 4: the original boundary references, representations and
// edge are removed.
fn retire_original(storage: &mut Storage, source: &SplitSource) {
    for reference in [source.start_ref, source.end_ref] {
        if reference.is_valid() {
            storage.mark_removed_ref(reference.into());
        }
    }
    storage.mark_rep_removed(source.curve.into());
    storage.mark_rep_removed(source.polygon.into());
    storage.mark_removed(source.edge.into());
}

// The child vertices of the original boundary references (`IsValid()` only, as
// natively).
fn boundary_vertices(storage: &Storage, source: &SplitSource) -> (VertexId, VertexId) {
    let child = |reference: VertexRefId| {
        if reference.is_valid() {
            storage.vertex_ref(reference).child_vertex_id
        } else {
            VertexId::default()
        }
    };
    (child(source.start_ref), child(source.end_ref))
}
