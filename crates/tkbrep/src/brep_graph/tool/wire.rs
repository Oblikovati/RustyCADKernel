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
// Rust translation of BRepGraph_Tool::Wire (ModelingData/TKBRep/BRepGraph/
// BRepGraph_Tool.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Tool::Wire`: the wire closure, sizes and owning face.
use super::{WireUsage, derived_state, face, wire_of_ref};
use crate::brep_graph::refs_iterator::{CoEdgesOfWire, RefsWireOfFace};
use crate::brep_graph::{BRepGraph, EdgeId, FaceId, WireId, WireRefId};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;

/// `Usage(theGraph, theWireRef)`: the active wire of an active reference with the
/// reference orientation; the invalid usage otherwise.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, WireRefId, tool};
/// assert!(!tool::wire::usage(&BRepGraph::new(), WireRefId::new(0)).is_valid());
/// ```
pub fn usage(graph: &BRepGraph, wire_ref: WireRefId) -> WireUsage {
    if !wire_ref.is_valid() || graph.refs().generic().is_removed(wire_ref.into()) {
        return WireUsage::default();
    }
    let entry = graph.inc_storage().wire_ref(wire_ref);
    if !entry.child_wire_id.is_valid() || entry.child_wire_id.is_removed(graph) {
        return WireUsage::default();
    }
    WireUsage {
        def_id: entry.child_wire_id,
        location: Location::default(),
        orientation: entry.orientation.orientation(),
    }
}

/// `IsClosed(theGraph, theWire)`: the ordered coedge chain returns to its start.
pub fn is_closed(graph: &BRepGraph, wire: WireId) -> bool {
    derived_state::wire_is_closed(graph, wire)
}

/// `IsClosed(theGraph, theWireRef)`.
pub fn is_closed_ref(graph: &BRepGraph, wire_ref: WireRefId) -> bool {
    let wire = wire_of_ref(graph, wire_ref);
    wire.is_valid() && is_closed(graph, wire)
}

/// `NbCoEdges(theGraph, theWire)`: the raw coedge count (seam halves twice).
pub fn nb_co_edges(graph: &BRepGraph, wire: WireId) -> u32 {
    graph.inc_storage().wire_relations(wire).co_edge_ids.len() as u32
}

/// `NbCoEdges(theGraph, theWireRef)`: 0 for an inactive reference or wire.
pub fn nb_co_edges_ref(graph: &BRepGraph, wire_ref: WireRefId) -> u32 {
    let wire = wire_of_ref(graph, wire_ref);
    if wire.is_valid() {
        nb_co_edges(graph, wire)
    } else {
        0
    }
}

/// `NbDistinctEdges(theGraph, theWire)`: the distinct edges of the active coedges.
pub fn nb_distinct_edges(graph: &BRepGraph, wire: WireId) -> u32 {
    let storage = graph.inc_storage();
    let mut seen: Vec<EdgeId> = Vec::new();
    let mut count = 0;
    for co_edge in CoEdgesOfWire::new(graph, wire) {
        let edge = storage.co_edge(co_edge).child_edge_id;
        if !seen.contains(&edge) {
            count += 1;
        }
        seen.push(edge);
    }
    count
}

/// `NbDistinctEdges(theGraph, theWireRef)`.
pub fn nb_distinct_edges_ref(graph: &BRepGraph, wire_ref: WireRefId) -> u32 {
    let wire = wire_of_ref(graph, wire_ref);
    if wire.is_valid() {
        nb_distinct_edges(graph, wire)
    } else {
        0
    }
}

/// `FaceOf(theGraph, theWire)`: the parent face of the first reference that is not
/// removed (the face itself is not checked); invalid for a free wire.
pub fn face_of(graph: &BRepGraph, wire: WireId) -> FaceId {
    let storage = graph.inc_storage();
    storage
        .wire_relations(wire)
        .parent_wire_ref_ids
        .iter()
        .find(|wire_ref| !wire_ref.is_removed(graph))
        .map_or(FaceId::default(), |wire_ref| {
            storage.wire_ref(*wire_ref).parent_face_id
        })
}

/// `FaceOf(theGraph, theWireRef)`.
pub fn face_of_ref(graph: &BRepGraph, wire_ref: WireRefId) -> FaceId {
    let wire = wire_of_ref(graph, wire_ref);
    if wire.is_valid() {
        face_of(graph, wire)
    } else {
        FaceId::default()
    }
}

/// `IsOuter(theGraph, theWire)`: at the first owning face that lists the wire through
/// an active reference, the face's outer wire is this wire.
pub fn is_outer(graph: &BRepGraph, wire: WireId) -> Result<bool, Failure> {
    let storage = graph.inc_storage();
    for wire_ref in &storage.wire_relations(wire).parent_wire_ref_ids {
        if wire_ref.is_removed(graph) {
            continue;
        }
        let owner = storage.wire_ref(*wire_ref).parent_face_id;
        if owner.is_valid() && face_lists_wire(graph, owner, wire) {
            return Ok(face::outer_wire(graph, owner)? == wire);
        }
    }
    Ok(false)
}

// The inner loop of `IsOuter`: an active reference of the face names the wire.
fn face_lists_wire(graph: &BRepGraph, owner: FaceId, wire: WireId) -> bool {
    RefsWireOfFace::new(graph, owner).any(|wire_ref| {
        !graph.refs().generic().is_removed(wire_ref.into())
            && graph.inc_storage().wire_ref(wire_ref).child_wire_id == wire
    })
}

/// `IsOuter(theGraph, theWireRef)`.
pub fn is_outer_ref(graph: &BRepGraph, wire_ref: WireRefId) -> Result<bool, Failure> {
    let wire = wire_of_ref(graph, wire_ref);
    if !wire.is_valid() {
        return Ok(false);
    }
    is_outer(graph, wire)
}
