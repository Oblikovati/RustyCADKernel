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
// Rust translation of checkGeometryReferences of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The representation-reference check of the audit.
use super::{Issue, Severity, is_entity_removed, is_valid_node_id, report};
use crate::brep_graph::iterator::{CoEdgeIterator, EdgeIterator, FaceIterator};
use crate::brep_graph::refs_iterator::RefsWireOfFace;
use crate::brep_graph::tool;
use crate::brep_graph::{BRepGraph, EdgeId, FaceId};
use crate::brep_graph_inc::{EdgeDef, FaceDef};

/// `checkGeometryReferences`: curves, surfaces, p-curves and mesh representation ids
/// are present where needed and in range.
pub(super) fn check_geometry_references(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    check_edge_representations(graph, issues);
    check_face_representations(graph, issues);
    check_unbounded_faces(graph, issues);
    check_co_edge_pcurves(graph, issues);
}

// `hasActiveSurface`: the face is live and its surface representation is live.
fn has_active_surface(graph: &BRepGraph, face: FaceId) -> bool {
    if !is_valid_node_id(graph, face) || is_entity_removed(graph, face) {
        return false;
    }
    let surface = graph.topo().faces().definition(face).surface_rep_id;
    surface.is_valid_for(graph.topo().geometry().nb_face_surfaces()) && !surface.is_removed(graph)
}

fn check_edge_representations(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        let (edge_id, edge) = (edges.current_id(), edges.current());
        check_edge_curve_presence(graph, edge_id, edge, issues);
        check_edge_rep_bounds(graph, edge_id, edge, issues);
        edges.next();
    }
}

// A non-degenerated edge needs a 3D curve unless every face it bounds has no surface.
fn check_edge_curve_presence(
    graph: &BRepGraph,
    edge_id: EdgeId,
    edge: &EdgeDef,
    issues: &mut Vec<Issue>,
) {
    if !needs_curve3d(graph, edge_id)
        || tool::edge::degenerated(graph, edge_id)
        || tool::edge::has_curve(graph, edge_id)
    {
        return;
    }
    let curve = edge.curve3d_rep_id;
    let text = if curve.is_valid() && curve.is_removed(graph) {
        "Non-degenerate EdgeDef references a removed Curve3DRep"
    } else {
        "Non-degenerate EdgeDef has no Curve3D representation"
    };
    report(issues, Severity::Error, edge_id, text);
}

// A free edge needs a curve; a face-bounding edge needs one when a face has a surface.
fn needs_curve3d(graph: &BRepGraph, edge: EdgeId) -> bool {
    let mut faces = graph.topo().edges().faces_of(edge);
    if !faces.more() {
        return true;
    }
    faces.any(|face| has_active_surface(graph, face))
}

fn check_edge_rep_bounds(
    graph: &BRepGraph,
    edge_id: EdgeId,
    edge: &EdgeDef,
    issues: &mut Vec<Issue>,
) {
    let curve = edge.curve3d_rep_id;
    if curve.is_valid() && !curve.is_valid_for(graph.topo().geometry().nb_edge_curves3d()) {
        report(
            issues,
            Severity::Error,
            edge_id,
            "EdgeDef.Curve3DRepId out of bounds",
        );
    }
    let polygon = edge.polygon3d_rep_id;
    if polygon.is_valid() && !polygon.is_valid_for(graph.mesh().poly().nb_edge_polygons3d()) {
        report(
            issues,
            Severity::Error,
            edge_id,
            "EdgeDef.Polygon3DRepId out of bounds",
        );
    }
}

fn check_face_representations(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut faces = FaceIterator::new(graph);
    while faces.more() {
        let face_id = faces.current_id();
        check_face_rep_bounds(graph, face_id, faces.current(), issues);
        // The cached mesh entry of a face must carry a triangulation.
        let cached = graph.mesh().cache().faces().entry(face_id);
        if cached.is_some_and(|entry| entry.triangulation.is_null()) {
            let text = "CacheMesh.FaceMesh.Triangulation is null";
            report(issues, Severity::Error, face_id, text);
        }
        faces.next();
    }
}

fn check_face_rep_bounds(
    graph: &BRepGraph,
    face_id: FaceId,
    face: &FaceDef,
    issues: &mut Vec<Issue>,
) {
    let surface = face.surface_rep_id;
    if surface.is_valid() && !surface.is_valid_for(graph.topo().geometry().nb_face_surfaces()) {
        report(
            issues,
            Severity::Error,
            face_id,
            "FaceDef.SurfaceRepId out of bounds",
        );
    }
    let triangulation = face.triangulation_rep_id;
    let nb_triangulations = graph.mesh().poly().nb_face_triangulations();
    if triangulation.is_valid() && !triangulation.is_valid_for(nb_triangulations) {
        let text = "FaceDef.TriangulationRepId out of bounds";
        report(issues, Severity::Error, face_id, text);
    }
}

// A surfaced face without wire references is allowed for intentionally unbounded
// natural faces, so it is only a warning the caller may accept.
fn check_unbounded_faces(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let nb_surfaces = graph.topo().geometry().nb_face_surfaces();
    let mut faces = FaceIterator::new(graph);
    while faces.more() {
        let surface = faces.current().surface_rep_id;
        let is_surfaced = surface.is_valid_for(nb_surfaces) && !surface.is_removed(graph);
        if is_surfaced && !RefsWireOfFace::new(graph, faces.current_id()).more() {
            let text = "Surfaced face has no wire refs";
            report(issues, Severity::Warning, faces.current_id(), text);
        }
        faces.next();
    }
}

// A coedge on a face with a live surface needs a p-curve.
fn check_co_edge_pcurves(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut co_edges = CoEdgeIterator::new(graph);
    while co_edges.more() {
        let co_edge_id = co_edges.current_id();
        let needs_pcurve = has_active_surface(graph, co_edges.current().face_id);
        if needs_pcurve && !tool::co_edge::has_pcurve(graph, co_edge_id) {
            let text = "CoEdgeDef has no Curve2D representation";
            report(issues, Severity::Error, co_edge_id, text);
        }
        co_edges.next();
    }
}
