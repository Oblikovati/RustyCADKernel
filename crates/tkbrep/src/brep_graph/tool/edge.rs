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
// Rust translation of BRepGraph_Tool::Edge and the coedge lookups of the anonymous
// namespace (ModelingData/TKBRep/BRepGraph/BRepGraph_Tool.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Tool::Edge`: the edge flags, range, curve, adaptors and coedge lookups.
//! The curve adaptors are in `edge_adaptors` and re-exported here.
use super::derived_state;
pub use super::edge_adaptors::{curve_adaptor, curve_adaptor_usage, curve_on_surface, curve_usage};
use crate::brep_graph::reverse_iterator::CoEdgesOfEdge;
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId, VertexId, VertexRefId};
use crate::brep_graph_inc::CoEdgeDef;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::Orientation;

/// `Tolerance(theGraph, theEdge)`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, tool};
/// # let graph = BRepGraph::new();
/// assert_eq!(tool::edge::nb_faces(&graph, Default::default()), 0);
/// ```
pub fn tolerance(graph: &BRepGraph, edge: EdgeId) -> f64 {
    graph.inc_storage().edge(edge).tolerance
}

/// `Degenerated(theGraph, theEdge)`: no 3D curve and collapsed vertices (derived from
/// the current geometry); false for an inactive edge.
pub fn degenerated(graph: &BRepGraph, edge: EdgeId) -> bool {
    derived_state::edge_is_degenerated(graph, edge)
}

/// `IsClosed(theGraph, theEdge)`: the same active vertex at both ends.
pub fn is_closed(graph: &BRepGraph, edge: EdgeId) -> bool {
    derived_state::edge_is_closed(graph, edge)
}

/// `Range(theGraph, theEdge)`: the parameter range of the active curve use, (0, 0)
/// otherwise.
pub fn range(graph: &BRepGraph, edge: EdgeId) -> (f64, f64) {
    let storage = graph.inc_storage();
    let rep = storage.edge(edge).curve3d_rep_id;
    if !rep.is_valid() || !rep.is_valid_for(storage.nb_edge_curves3d()) || storage.is_removed(rep) {
        return (0.0, 0.0);
    }
    let record = storage.edge_curve3d_rep(rep);
    (record.param_first, record.param_last)
}

/// `StartVertexId(theGraph, theEdge)`: the start vertex reference.
pub fn start_vertex_id(graph: &BRepGraph, edge: EdgeId) -> VertexRefId {
    graph.inc_storage().edge(edge).start_vertex_ref_id
}

/// `EndVertexId(theGraph, theEdge)`: the end vertex reference.
pub fn end_vertex_id(graph: &BRepGraph, edge: EdgeId) -> VertexRefId {
    graph.inc_storage().edge(edge).end_vertex_ref_id
}

/// `HasCurve(theGraph, theEdge)`: `Topo().Edges().Curve3D` is not null.
pub fn has_curve(graph: &BRepGraph, edge: EdgeId) -> bool {
    graph.topo().edges().curve3d(edge).is_some()
}

/// `Curve(theGraph, theEdge)`: the 3D curve of an active curve use, null otherwise
/// (the edge itself must be in range; its removal is not checked).
pub fn curve(graph: &BRepGraph, edge: EdgeId) -> Handle<dyn GeomCurve> {
    let storage = graph.inc_storage();
    let rep = storage.edge(edge).curve3d_rep_id;
    if !rep.is_valid_for(storage.nb_edge_curves3d()) || storage.is_removed(rep) {
        return Handle::default();
    }
    storage.edge_curve3d_rep(rep).curve.clone()
}

/// `FindByVertices(theGraph, theStart, theEnd, theToIgnoreOrientation)`: the first
/// active edge of the start vertex whose active vertex references name the two
/// vertices (in either order when orientation is ignored).
pub fn find_by_vertices(
    graph: &BRepGraph,
    start: VertexId,
    end: VertexId,
    ignore_orientation: bool,
) -> EdgeId {
    let storage = graph.inc_storage();
    let nb_vertices = storage.nb_vertices();
    if !start.is_valid_for(nb_vertices)
        || !end.is_valid_for(nb_vertices)
        || start.is_removed(graph)
        || end.is_removed(graph)
    {
        return EdgeId::default();
    }
    graph
        .topo()
        .vertices()
        .edges(start)
        .iter()
        .copied()
        .find(|edge| edge_joins(graph, *edge, (start, end), ignore_orientation))
        .unwrap_or_default()
}

// The loop body of `FindByVertices`: an active edge with active references whose
// vertices match.
fn edge_joins(
    graph: &BRepGraph,
    edge: EdgeId,
    (start, end): (VertexId, VertexId),
    ignore_orientation: bool,
) -> bool {
    let storage = graph.inc_storage();
    if !edge.is_valid_for(storage.nb_edges()) || edge.is_removed(graph) {
        return false;
    }
    let def = storage.edge(edge);
    let nb_refs = storage.nb_vertex_refs();
    let (start_ref, end_ref) = (def.start_vertex_ref_id, def.end_vertex_ref_id);
    if !start_ref.is_valid_for(nb_refs)
        || !end_ref.is_valid_for(nb_refs)
        || start_ref.is_removed(graph)
        || end_ref.is_removed(graph)
    {
        return false;
    }
    let first = storage.vertex_ref(start_ref).child_vertex_id;
    let last = storage.vertex_ref(end_ref).child_vertex_id;
    (first == start && last == end) || (ignore_orientation && first == end && last == start)
}

/// The content a coedge lookup requires (`CoEdgeLookupContent`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum LookupContent {
    Any,
    WithPCurve,
}

// `matchesLookupContent`: any coedge, or one with an active p-curve use.
fn matches_content(graph: &BRepGraph, co_edge: &CoEdgeDef, content: LookupContent) -> bool {
    let rep = co_edge.curve2d_rep_id;
    content == LookupContent::Any
        || (rep.is_valid_for(graph.inc_storage().nb_co_edge_curves2d()) && !rep.is_removed(graph))
}

// The active coedges of the edge on the face with the required content.
fn candidate_co_edges<'g>(
    graph: &'g BRepGraph,
    edge: EdgeId,
    face: FaceId,
    content: LookupContent,
) -> impl Iterator<Item = (CoEdgeId, &'g CoEdgeDef)> + 'g {
    let mut iterator = CoEdgesOfEdge::new(graph, graph.topo().edges().co_edges(edge));
    std::iter::from_fn(move || {
        while iterator.more() {
            let found = (iterator.current_id(), iterator.definition());
            iterator.next();
            if found.1.child_edge_id == edge
                && found.1.face_id == face
                && matches_content(graph, found.1, content)
            {
                return Some(found);
            }
        }
        None
    })
}

// `findCoEdgeId(graph, edge, face, content)`: the first candidate.
fn find_first(graph: &BRepGraph, edge: EdgeId, face: FaceId, content: LookupContent) -> CoEdgeId {
    if !edge.is_valid_for(graph.topo().edges().nb())
        || !face.is_valid_for(graph.topo().faces().nb())
    {
        return CoEdgeId::default();
    }
    candidate_co_edges(graph, edge, face, content)
        .map(|(id, _)| id)
        .next()
        .unwrap_or_default()
}

// `findCoEdgeId(graph, edge, face, orientation, content)`: the first candidate with the
// orientation, else the first candidate.
fn find_oriented(
    graph: &BRepGraph,
    (edge, face): (EdgeId, FaceId),
    orientation: Orientation,
    content: LookupContent,
) -> CoEdgeId {
    if !edge.is_valid_for(graph.topo().edges().nb())
        || !face.is_valid_for(graph.topo().faces().nb())
    {
        return CoEdgeId::default();
    }
    let mut first_match = CoEdgeId::default();
    for (id, def) in candidate_co_edges(graph, edge, face, content) {
        if !first_match.is_valid() {
            first_match = id;
        }
        if def.orientation.orientation() == orientation {
            return id;
        }
    }
    first_match
}

/// `FindPCurveCoEdgeId(theGraph, theEdge, theFace)`: the first active coedge of the
/// edge-face use with an active p-curve.
pub fn find_pcurve_co_edge_id(graph: &BRepGraph, edge: EdgeId, face: FaceId) -> CoEdgeId {
    find_first(graph, edge, face, LookupContent::WithPCurve)
}

/// `FindPCurveCoEdgeId(theGraph, theEdge, theFace, theOrientation)`: the p-curve coedge
/// with the orientation, else the first p-curve coedge of the use.
pub fn find_pcurve_co_edge_id_oriented(
    graph: &BRepGraph,
    edge: EdgeId,
    face: FaceId,
    orientation: Orientation,
) -> CoEdgeId {
    find_oriented(graph, (edge, face), orientation, LookupContent::WithPCurve)
}

/// `FindCoEdgeId(theGraph, theEdge, theFace)`: the first active coedge of the use.
pub fn find_co_edge_id(graph: &BRepGraph, edge: EdgeId, face: FaceId) -> CoEdgeId {
    find_first(graph, edge, face, LookupContent::Any)
}

/// `FindCoEdgeId(theGraph, theEdge, theFace, theOrientation)`: the coedge with the
/// orientation, else the first coedge of the use.
pub fn find_co_edge_id_oriented(
    graph: &BRepGraph,
    edge: EdgeId,
    face: FaceId,
    orientation: Orientation,
) -> CoEdgeId {
    find_oriented(graph, (edge, face), orientation, LookupContent::Any)
}

/// `NbFaces(theGraph, theEdge)`: `Topo().Edges().NbFaces`.
pub fn nb_faces(graph: &BRepGraph, edge: EdgeId) -> u32 {
    graph.topo().edges().nb_faces(edge)
}

/// `IsManifold(theGraph, theEdge)`: exactly two faces.
pub fn is_manifold(graph: &BRepGraph, edge: EdgeId) -> bool {
    nb_faces(graph, edge) == 2
}

/// `IsBoundary(theGraph, theEdge)`: exactly one face.
pub fn is_boundary(graph: &BRepGraph, edge: EdgeId) -> bool {
    nb_faces(graph, edge) == 1
}

/// `IsSeamOnFace(theGraph, theEdge, theFace)`: two active coedges of the edge on the
/// face with different orientations (the coedge's own edge is not compared).
pub fn is_seam_on_face(graph: &BRepGraph, edge: EdgeId, face: FaceId) -> bool {
    if !edge.is_valid() || !face.is_valid() {
        return false;
    }
    let mut first: Option<Orientation> = None;
    let mut co_edges = CoEdgesOfEdge::new(graph, graph.topo().edges().co_edges(edge));
    while co_edges.more() {
        let def = co_edges.definition();
        co_edges.next();
        if def.face_id != face {
            continue;
        }
        let orientation = def.orientation.orientation();
        match first {
            None => first = Some(orientation),
            Some(seen) if seen != orientation => return true,
            Some(_) => {}
        }
    }
    false
}
