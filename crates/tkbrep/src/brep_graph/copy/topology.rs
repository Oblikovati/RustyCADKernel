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
// Rust translation of `ensureVertex`, `ensureEdge`, `ensureWire` and `ensureFace` of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Copy.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The memoised copies of vertices, edges, wires and faces.
//!
//! Each `ensure*` reads what it needs from the source before it writes the result: the
//! native references into the source stores stay valid across the result's appends
//! (`NCollection_DynamicArray` never relocates), and nothing a copy writes changes a
//! source record, so the early reads see what the native late reads see.
use super::context::CopyContext;
use super::meshes::{ensure_polygon3d_rep, ensure_triangulation_rep};
use super::pcurves::ensure_pcurves_for_face;
use super::policy::{MeshPolicy, copy_geometry};
use crate::brep_graph::refs_iterator::{CoEdgesOfWire, RefsWireOfFace};
use crate::brep_graph::tool;
use crate::brep_graph::{
    BRepGraph, CoEdgeId, EdgeId, FaceId, ItemId, RefId, VertexId, VertexRefId, WireId,
};
use rustycad_tkernel::standard::Failure;

/// `ensureVertex(theCtx, theSrcId)`: the copy of a source vertex (invalid for an id past
/// the source vertices).
pub(crate) fn ensure_vertex(
    context: &mut CopyContext<'_>,
    source: VertexId,
) -> Result<VertexId, Failure> {
    if !source.is_valid_in(&context.source().topo().vertices()) {
        return Ok(VertexId::invalid());
    }
    // An external copy may find the vertex already copied; a self-copy starts empty.
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let def = context.source().topo().vertices().definition(source);
    let (point, tolerance) = (def.point, def.tolerance);
    let copied = context
        .target_mut()
        .editor()
        .vertices()
        .add(point, tolerance);
    context.copy_removed(source, copied);
    context.bind_node(source, copied);
    Ok(copied)
}

// The child vertex of an edge's vertex reference, invalid without one.
fn vertex_of(graph: &BRepGraph, reference: VertexRefId) -> VertexId {
    if !reference.is_valid() {
        return VertexId::invalid();
    }
    graph.refs().vertices().entry(reference).child_vertex_id
}

/// `ensureEdge(theCtx, theSrcId)`: the copy of a source edge with its vertices, 3D curve
/// (under the geometry policy) and polygon; the vertex references follow the edge's.
pub(crate) fn ensure_edge(context: &mut CopyContext<'_>, source: EdgeId) -> Result<EdgeId, Failure> {
    if !source.is_valid_in(&context.source().topo().edges()) {
        return Ok(EdgeId::invalid());
    }
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let graph = context.source();
    let edge = graph.topo().edges().definition(source).clone();
    let start = vertex_of(graph, tool::edge::start_vertex_id(graph, source));
    let end = vertex_of(graph, tool::edge::end_vertex_id(graph, source));
    let curve = tool::edge::curve(graph, source);
    let (first, last) = tool::edge::range(graph, source);
    let new_start = ensure_vertex(context, start)?;
    let new_end = ensure_vertex(context, end)?;
    let curve = copy_geometry(&curve, context.geom)?;
    let copied = context.target_mut().editor().edges().add(
        new_start,
        new_end,
        &curve,
        first,
        last,
        edge.tolerance,
    );
    context.copy_removed(source, copied);
    if context.mesh != MeshPolicy::Drop && edge.polygon3d_rep_id.is_valid() {
        ensure_polygon3d_rep(context, edge.polygon3d_rep_id, copied)?;
    }
    context.bind_node(source, copied);
    let new_edge = context.target().topo().edges().definition(copied);
    let refs = [
        (edge.start_vertex_ref_id, new_edge.start_vertex_ref_id),
        (edge.end_vertex_ref_id, new_edge.end_vertex_ref_id),
    ];
    for (old_ref, new_ref) in refs {
        if old_ref.is_valid() && new_ref.is_valid() {
            context.copy_removed(old_ref, new_ref);
        }
    }
    for (old_ref, new_ref) in refs {
        context.bind_item(ref_item(old_ref), ref_item(new_ref));
    }
    Ok(copied)
}

/// The item of a typed reference id.
pub(crate) fn ref_item(reference: impl Into<RefId>) -> ItemId {
    ItemId::from(reference.into())
}

/// `ensureWire(theCtx, theSrcId)`: the copy of a source wire: each coedge's edge, a new
/// free coedge per use, then the wire over them; the coedges bind after the wire.
pub(crate) fn ensure_wire(context: &mut CopyContext<'_>, source: WireId) -> Result<WireId, Failure> {
    if !source.is_valid_in(&context.source().topo().wires()) {
        return Ok(WireId::invalid());
    }
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let source_co_edges: Vec<CoEdgeId> = CoEdgesOfWire::new(context.source(), source).collect();
    let mut new_co_edges = Vec::with_capacity(source_co_edges.len());
    for &co_edge in &source_co_edges {
        let def = context.source().topo().co_edges().definition(co_edge);
        let (edge, orientation) = (def.child_edge_id, def.orientation);
        let new_edge = ensure_edge(context, edge)?;
        let copied = context
            .target_mut()
            .editor()
            .co_edges()
            .add(new_edge, orientation);
        context.copy_removed(co_edge, copied);
        new_co_edges.push(copied);
    }
    let copied = context.target_mut().editor().wires().add(&new_co_edges);
    context.copy_removed(source, copied);
    context.bind_node(source, copied);
    for (old, new) in source_co_edges.into_iter().zip(new_co_edges) {
        context.bind_node(old, new);
    }
    Ok(copied)
}

/// `ensureFace(theCtx, theSrcId)`: the copy of a source face: its wires, surface (under
/// the geometry policy) and triangulation, the wire references following the source's
/// pairwise, then the p-curves of its coedges.
pub(crate) fn ensure_face(context: &mut CopyContext<'_>, source: FaceId) -> Result<FaceId, Failure> {
    if !source.is_valid_in(&context.source().topo().faces()) {
        return Ok(FaceId::invalid());
    }
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let graph = context.source();
    let face = graph.topo().faces().definition(source).clone();
    let wires: Vec<WireId> = RefsWireOfFace::new(graph, source)
        .map(|reference| graph.refs().wires().entry(reference).child_wire_id)
        .collect();
    let mut outer = WireId::invalid();
    let mut inner = Vec::new();
    for wire in wires {
        let copied = ensure_wire(context, wire)?;
        if !outer.is_valid() {
            outer = copied;
        } else {
            inner.push(copied);
        }
    }
    let surface = tool::face::surface(context.source(), source);
    let surface = copy_geometry(&surface, context.geom)?;
    let copied =
        context
            .target_mut()
            .editor()
            .faces()
            .add(&surface, outer, &inner, face.tolerance);
    context.copy_removed(source, copied);
    if context.mesh != MeshPolicy::Drop && face.triangulation_rep_id.is_valid() {
        ensure_triangulation_rep(context, face.triangulation_rep_id, copied)?;
    }
    context.bind_node(source, copied);
    bind_wire_refs(context, source, copied);
    // PCurves require both the face and its edges to exist in the result.
    ensure_pcurves_for_face(context, source, copied)?;
    Ok(copied)
}

// The wire references of the source and copied faces, paired up to the shorter list:
// removed flags and item remap follow. A face the editor refused (a removed wire) is
// the invalid id, whose relations the native code reads out of bounds; it has none here.
fn bind_wire_refs(context: &mut CopyContext<'_>, source: FaceId, copied: FaceId) {
    let old_refs = context.source().topo().faces().relations(source).wire_ref_ids.clone();
    let target = context.target();
    if !copied.is_valid_in(&target.topo().faces()) {
        return;
    }
    let new_refs = target.topo().faces().relations(copied).wire_ref_ids.clone();
    for (old, new) in old_refs.into_iter().zip(new_refs) {
        context.copy_removed(old, new);
        context.bind_item(ref_item(old), ref_item(new));
    }
}
