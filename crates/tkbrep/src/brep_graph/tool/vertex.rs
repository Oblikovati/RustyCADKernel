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
// Rust translation of BRepGraph_Tool::Vertex (ModelingData/TKBRep/BRepGraph/
// BRepGraph_Tool.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Tool::Vertex`: the vertex point and tolerance.
use super::VertexUsage;
use crate::brep_graph::{BRepGraph, VertexId, VertexRefId};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

/// `Usage(theGraph, theVertexRef)`: the active vertex of an active reference with the
/// reference orientation and the identity location; the invalid usage otherwise.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, VertexRefId, tool};
/// assert!(!tool::vertex::usage(&BRepGraph::new(), VertexRefId::new(0)).is_valid());
/// ```
pub fn usage(graph: &BRepGraph, vertex_ref: VertexRefId) -> VertexUsage {
    if !vertex_ref.is_valid() || graph.refs().generic().is_removed(vertex_ref.into()) {
        return VertexUsage::default();
    }
    let entry = graph.inc_storage().vertex_ref(vertex_ref);
    if !entry.child_vertex_id.is_valid() || entry.child_vertex_id.is_removed(graph) {
        return VertexUsage::default();
    }
    VertexUsage {
        def_id: entry.child_vertex_id,
        location: Location::default(),
        orientation: entry.orientation.orientation(),
    }
}

/// `Pnt(theGraph, VertexUsage)`: the point moved by the usage location.
pub fn pnt_usage(graph: &BRepGraph, usage: &VertexUsage) -> Point3 {
    let point = graph.inc_storage().vertex(usage.def_id).point;
    if usage.location.is_identity() {
        return point;
    }
    point.transformed(usage.location.transformation())
}

/// `Pnt(theGraph, theVertex)`: the point in the definition frame.
pub fn pnt(graph: &BRepGraph, vertex: VertexId) -> Point3 {
    graph.inc_storage().vertex(vertex).point
}

/// `Pnt(theGraph, theVertexRef)`: the point of the reference's usage, the origin for
/// an inactive reference.
pub fn pnt_ref(graph: &BRepGraph, vertex_ref: VertexRefId) -> Point3 {
    let vertex_usage = usage(graph, vertex_ref);
    if vertex_usage.is_valid() {
        pnt_usage(graph, &vertex_usage)
    } else {
        Point3::default()
    }
}

/// `Tolerance(theGraph, theVertex)`.
pub fn tolerance(graph: &BRepGraph, vertex: VertexId) -> f64 {
    graph.inc_storage().vertex(vertex).tolerance
}

/// `Tolerance(theGraph, theVertexRef)`: 0 for an inactive reference or vertex.
pub fn tolerance_ref(graph: &BRepGraph, vertex_ref: VertexRefId) -> f64 {
    if !vertex_ref.is_valid() || graph.refs().generic().is_removed(vertex_ref.into()) {
        return 0.0;
    }
    let vertex = graph.inc_storage().vertex_ref(vertex_ref).child_vertex_id;
    if vertex.is_valid() && !vertex.is_removed(graph) {
        tolerance(graph, vertex)
    } else {
        0.0
    }
}

/// `NbEdges(theGraph, theVertex)`: the size of the vertex's edge relation (empty for
/// an id out of range).
pub fn nb_edges(graph: &BRepGraph, vertex: VertexId) -> u32 {
    graph.topo().vertices().edges(vertex).len() as u32
}
