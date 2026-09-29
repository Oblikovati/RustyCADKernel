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
// Rust translation of the static members and the compute functions of
// ModelingData/TKBRep/BRepGraph/BRepGraph_CacheDerivedState.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The derived edge, wire and shell state of `BRepGraph_CacheDerivedState`, computed
//! from the current graph.
//!
//! The static `ComputeEdgeProperties`, `ComputeWireIsClosed` and `ComputeShellIsClosed`
//! members and the per-coedge SameRange/SameParameter computations are free functions
//! here; the cache service [`CacheDerivedState`] memoises them per OwnGen
//! (`IsDegenerated`, `SameParameter`, `SameRange`...).
use crate::brep_graph::tool;
use crate::brep_graph::*;
use crate::brep_graph_inc::EdgeDef;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use std::collections::BTreeMap;

mod copy;
mod entries;
mod service;

pub use service::CacheDerivedState;

/// `EdgeEntry::GeomStatus`: the geometric status of an edge (the native `Invalid`
/// value is never stored).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EdgeGeomStatus {
    /// `HasCurve3D`.
    HasCurve3D,
    /// `DegenerateOnSurface`: no 3D curve and collapsed vertices.
    DegenerateOnSurface,
    /// `MissingCurve3D`.
    MissingCurve3D,
}

/// `ShellEntry::ClosureStatus`: the closure of a shell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ShellClosureStatus {
    /// `Empty`.
    Empty,
    /// `Open`: an edge used once.
    Open,
    /// `Closed`.
    Closed,
    /// `NonManifold`: an edge used more than twice.
    NonManifold,
    /// `Invalid`: the shell is invalid or removed (also the unset cache value).
    #[default]
    Invalid,
}

// `resolveChildVertex`: the active vertex of an active vertex reference.
fn resolve_child_vertex(graph: &BRepGraph, reference: VertexRefId) -> VertexId {
    let storage = graph.inc_storage();
    if !reference.is_valid_for(storage.nb_vertex_refs()) || reference.is_removed(graph) {
        return VertexId::invalid();
    }
    let vertex = storage.vertex_ref(reference).child_vertex_id;
    if vertex.is_valid_for(storage.nb_vertices()) && !vertex.is_removed(graph) {
        vertex
    } else {
        VertexId::invalid()
    }
}

/// `computeStatusOnly`: the status and the closure (same start and end vertex) of an
/// edge in range.
pub fn edge_status(graph: &BRepGraph, edge: EdgeId) -> (EdgeGeomStatus, bool) {
    let def = graph.inc_storage().edge(edge);
    let start = resolve_child_vertex(graph, def.start_vertex_ref_id);
    let end = resolve_child_vertex(graph, def.end_vertex_ref_id);
    let closed = start.is_valid() && start == end;
    let status = if tool::edge::curve(graph, edge).is_some() {
        EdgeGeomStatus::HasCurve3D
    } else if closed {
        EdgeGeomStatus::DegenerateOnSurface
    } else if start.is_valid() && end.is_valid() {
        let storage = graph.inc_storage();
        let distance = storage
            .vertex(start)
            .point
            .distance(&storage.vertex(end).point);
        if distance <= def.tolerance {
            EdgeGeomStatus::DegenerateOnSurface
        } else {
            EdgeGeomStatus::MissingCurve3D
        }
    } else {
        EdgeGeomStatus::MissingCurve3D
    };
    (status, closed)
}

/// `ComputeEdgeProperties(theGraph, theEdge, theIsDegenerated, theIsClosed)`: `None`
/// for an edge out of range or removed, else (degenerated, closed).
pub fn compute_edge_properties(graph: &BRepGraph, edge: EdgeId) -> Option<(bool, bool)> {
    if !edge.is_valid_for(graph.inc_storage().nb_edges()) || edge.is_removed(graph) {
        return None;
    }
    let (status, closed) = edge_status(graph, edge);
    Some((status == EdgeGeomStatus::DegenerateOnSurface, closed))
}

// The active edge of an active coedge.
fn active_edge_of(graph: &BRepGraph, co_edge: CoEdgeId) -> Option<EdgeId> {
    let storage = graph.inc_storage();
    if !co_edge.is_valid_for(storage.nb_co_edges()) || co_edge.is_removed(graph) {
        return None;
    }
    let edge = storage.co_edge(co_edge).child_edge_id;
    (edge.is_valid_for(storage.nb_edges()) && !edge.is_removed(graph)).then_some(edge)
}

// The SameRange/SameParameter value of an edge without 3D curve: a closed edge.
fn closed_without_curve(graph: &BRepGraph, def: &EdgeDef) -> bool {
    let start = resolve_child_vertex(graph, def.start_vertex_ref_id);
    let end = resolve_child_vertex(graph, def.end_vertex_ref_id);
    start.is_valid() && start == end
}

// The active face of a coedge, or `None`.
fn active_face_of(graph: &BRepGraph, co_edge: CoEdgeId) -> Option<FaceId> {
    let face = graph.inc_storage().co_edge(co_edge).face_id;
    (face.is_valid_for(graph.inc_storage().nb_faces()) && !face.is_removed(graph)).then_some(face)
}

/// `computeSameRange`: an inactive coedge or edge is same-range; an edge with a 3D curve
/// compares the coedge's p-curve range with the edge range within PConfusion; an edge
/// without one is same-range when it is closed.
pub fn compute_same_range(graph: &BRepGraph, co_edge: CoEdgeId) -> bool {
    let Some(edge) = active_edge_of(graph, co_edge) else {
        return true;
    };
    if tool::edge::curve(graph, edge).is_null() {
        return closed_without_curve(graph, graph.inc_storage().edge(edge));
    }
    is_co_edge_same_range(graph, co_edge, edge)
}

// `isCoEdgeSameRange`.
fn is_co_edge_same_range(graph: &BRepGraph, co_edge: CoEdgeId, edge: EdgeId) -> bool {
    if active_face_of(graph, co_edge).is_none() || tool::co_edge::pcurve(graph, co_edge).is_null() {
        return true;
    }
    let edge_range = tool::edge::range(graph, edge);
    let pcurve_range = tool::co_edge::range(graph, co_edge);
    (pcurve_range.0 - edge_range.0).abs() <= precision::p_confusion()
        && (pcurve_range.1 - edge_range.1).abs() <= precision::p_confusion()
}

/// `computeSameParameter`: as SameRange for the inactive and curve-less cases; an edge
/// with a 3D curve is same-parameter when it is same-range and six samples of the 3D
/// curve lie within the edge tolerance (plus Confusion) of the surface points of the
/// p-curve. The adaptor loads raise as natively (a reversed range).
pub fn compute_same_parameter(graph: &BRepGraph, co_edge: CoEdgeId) -> Result<bool, Failure> {
    same_parameter_with_range(graph, co_edge, || compute_same_range(graph, co_edge))
}

/// The SameParameter computation given the coedge's SameRange answer (read only when
/// the edge has a 3D curve): the cache service passes the SameRange flag of its entry,
/// as `computeSameParameter` does.
pub(crate) fn same_parameter_with_range(
    graph: &BRepGraph,
    co_edge: CoEdgeId,
    same_range: impl FnOnce() -> bool,
) -> Result<bool, Failure> {
    let Some(edge) = active_edge_of(graph, co_edge) else {
        return Ok(true);
    };
    let def = graph.inc_storage().edge(edge);
    let curve = tool::edge::curve(graph, edge);
    if curve.is_null() {
        return Ok(closed_without_curve(graph, def));
    }
    if !same_range() {
        return Ok(false);
    }
    is_co_edge_same_parameter(graph, co_edge, def, &curve, tool::edge::range(graph, edge))
}

// `isCoEdgeSameParameter`.
fn is_co_edge_same_parameter(
    graph: &BRepGraph,
    co_edge: CoEdgeId,
    def: &EdgeDef,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
) -> Result<bool, Failure> {
    let Some(face) = active_face_of(graph, co_edge) else {
        return Ok(true);
    };
    let pcurve = tool::co_edge::pcurve(graph, co_edge);
    let surface = tool::face::surface(graph, face);
    if pcurve.is_null() || surface.is_null() {
        return Ok(true);
    }
    let tolerance = def.tolerance + precision::confusion();
    const NB_SAMPLES: i32 = 5;
    let curve_adaptor = GeomAdaptorCurve::with_range(curve, range.0, range.1)?;
    let pcurve_adaptor = Geom2dAdaptorCurve::with_curve(&pcurve)?;
    let surface_adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    for index in 0..=NB_SAMPLES {
        let parameter = range.0 + (range.1 - range.0) * f64::from(index) / f64::from(NB_SAMPLES);
        let point = curve_adaptor.eval_d0(parameter)?;
        let uv = pcurve_adaptor.eval_d0(parameter)?;
        let on_surface = surface_adaptor.eval_d0(uv.x(), uv.y())?;
        if point.distance(&on_surface) > tolerance {
            return Ok(false);
        }
    }
    Ok(true)
}

// `coEdgeOrientedVertices` of the derived state: the resolved start and end vertices of
// an active coedge in its orientation.
fn oriented_vertices(graph: &BRepGraph, co_edge: CoEdgeId) -> Option<(VertexId, VertexId)> {
    let edge = active_edge_of(graph, co_edge)?;
    let storage = graph.inc_storage();
    let def = storage.edge(edge);
    let reversed = storage.co_edge(co_edge).orientation.is_reversed;
    let (start_ref, end_ref) = if reversed {
        (def.end_vertex_ref_id, def.start_vertex_ref_id)
    } else {
        (def.start_vertex_ref_id, def.end_vertex_ref_id)
    };
    let start = resolve_child_vertex(graph, start_ref);
    let end = resolve_child_vertex(graph, end_ref);
    (start.is_valid() && end.is_valid()).then_some((start, end))
}

/// `ComputeWireIsClosed(theGraph, theWire)`: an active, non-empty wire whose coedges
/// chain end to start and return to the first start.
pub fn compute_wire_is_closed(graph: &BRepGraph, wire: WireId) -> bool {
    let storage = graph.inc_storage();
    if !wire.is_valid_for(storage.nb_wires()) || wire.is_removed(graph) {
        return false;
    }
    let co_edges = &storage.wire_relations(wire).co_edge_ids;
    let mut chain: Option<(VertexId, VertexId)> = None;
    for co_edge in co_edges {
        let Some((start, end)) = oriented_vertices(graph, *co_edge) else {
            return false;
        };
        chain = match chain {
            None => Some((start, end)),
            Some((_, previous_end)) if start != previous_end => return false,
            Some((first_start, _)) => Some((first_start, end)),
        };
    }
    chain.is_some_and(|(first_start, last_end)| last_end == first_start)
}

/// `computeShellClosure`: the use count of every non-degenerate edge of the active
/// faces' active wires (coedges of that face only) decides the closure.
pub fn compute_shell_closure(graph: &BRepGraph, shell: ShellId) -> ShellClosureStatus {
    let storage = graph.inc_storage();
    if !shell.is_valid_for(storage.nb_shells()) || shell.is_removed(graph) {
        return ShellClosureStatus::Invalid;
    }
    if storage.shell_relations(shell).face_ref_ids.is_empty() {
        return ShellClosureStatus::Empty;
    }
    let faces = active_shell_faces(graph, shell);
    if faces.is_empty() {
        return ShellClosureStatus::Empty;
    }
    let mut usage: BTreeMap<EdgeId, u32> = BTreeMap::new();
    for face in faces {
        for edge in face_boundary_edges(graph, face) {
            *usage.entry(edge).or_insert(0) += 1;
        }
    }
    classify_edge_usage(&usage)
}

// The distinct active faces of the shell's active face references.
fn active_shell_faces(graph: &BRepGraph, shell: ShellId) -> Vec<FaceId> {
    let storage = graph.inc_storage();
    let mut faces: Vec<FaceId> = Vec::new();
    for reference in &storage.shell_relations(shell).face_ref_ids {
        if !reference.is_valid_for(storage.nb_face_refs()) || reference.is_removed(graph) {
            continue;
        }
        let face = storage.face_ref(*reference).child_face_id;
        if face.is_valid_for(storage.nb_faces())
            && !face.is_removed(graph)
            && !faces.contains(&face)
        {
            faces.push(face);
        }
    }
    faces
}

// The non-degenerate active edges of the face's coedges, one entry per coedge use.
fn face_boundary_edges(graph: &BRepGraph, face: FaceId) -> Vec<EdgeId> {
    let storage = graph.inc_storage();
    let mut edges = Vec::new();
    for reference in &storage.face_relations(face).wire_ref_ids {
        if !reference.is_valid_for(storage.nb_wire_refs()) || reference.is_removed(graph) {
            continue;
        }
        let wire = storage.wire_ref(*reference).child_wire_id;
        if !wire.is_valid_for(storage.nb_wires()) || wire.is_removed(graph) {
            continue;
        }
        for co_edge in &storage.wire_relations(wire).co_edge_ids {
            if let Some(edge) = face_co_edge_edge(graph, face, *co_edge) {
                edges.push(edge);
            }
        }
    }
    edges
}

fn face_co_edge_edge(graph: &BRepGraph, face: FaceId, co_edge: CoEdgeId) -> Option<EdgeId> {
    let storage = graph.inc_storage();
    if !co_edge.is_valid_for(storage.nb_co_edges()) || co_edge.is_removed(graph) {
        return None;
    }
    let def = storage.co_edge(co_edge);
    if def.face_id != face {
        return None;
    }
    let edge = def.child_edge_id;
    if !edge.is_valid_for(storage.nb_edges()) || edge.is_removed(graph) {
        return None;
    }
    (edge_status(graph, edge).0 != EdgeGeomStatus::DegenerateOnSurface).then_some(edge)
}

fn classify_edge_usage(usage: &BTreeMap<EdgeId, u32>) -> ShellClosureStatus {
    if usage.is_empty() {
        return ShellClosureStatus::Closed;
    }
    if usage.values().any(|count| *count > 2) {
        return ShellClosureStatus::NonManifold;
    }
    if usage.values().any(|count| *count == 1) {
        return ShellClosureStatus::Open;
    }
    ShellClosureStatus::Closed
}

/// `ComputeShellIsClosed(theGraph, theShell)`.
pub fn compute_shell_is_closed(graph: &BRepGraph, shell: ShellId) -> bool {
    compute_shell_closure(graph, shell) == ShellClosureStatus::Closed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph_inc::ParityOrientation;

    fn closed_edge(graph: &mut BRepGraph) -> EdgeId {
        let storage = graph.inc_storage_mut();
        let (edge, vertex) = (storage.append_edge(), storage.append_vertex());
        for start in [true, false] {
            let reference = storage.append_vertex_ref();
            storage.change_vertex_ref(reference).child_vertex_id = vertex;
            if start {
                storage.change_edge(edge).start_vertex_ref_id = reference;
            } else {
                storage.change_edge(edge).end_vertex_ref_id = reference;
            }
        }
        edge
    }

    #[test]
    fn curveless_closed_edges_are_degenerate() {
        let mut graph = BRepGraph::new();
        let edge = closed_edge(&mut graph);
        assert_eq!(compute_edge_properties(&graph, edge), Some((true, true)));
        assert_eq!(compute_edge_properties(&graph, EdgeId::new(4)), None);
        let storage = graph.inc_storage_mut();
        let (face, wire) = (storage.append_face(), storage.append_wire());
        let co_edge = storage.create_co_edge_use(wire, edge, face, ParityOrientation::FORWARD);
        assert!(compute_same_range(&graph, co_edge));
        assert!(compute_same_parameter(&graph, co_edge).unwrap());
        assert!(compute_wire_is_closed(&graph, wire));
        assert!(!compute_wire_is_closed(&graph, WireId::new(9)));
    }

    #[test]
    fn shell_closure_counts_edge_uses() {
        let mut usage = BTreeMap::new();
        assert_eq!(classify_edge_usage(&usage), ShellClosureStatus::Closed);
        usage.insert(EdgeId::new(0), 1);
        assert_eq!(classify_edge_usage(&usage), ShellClosureStatus::Open);
        usage.insert(EdgeId::new(1), 3);
        assert_eq!(classify_edge_usage(&usage), ShellClosureStatus::NonManifold);
        let graph = BRepGraph::new();
        assert_eq!(
            compute_shell_closure(&graph, ShellId::new(0)),
            ShellClosureStatus::Invalid
        );
        assert!(!compute_shell_is_closed(&graph, ShellId::new(0)));
    }
}
