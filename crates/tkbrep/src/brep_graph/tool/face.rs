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
// Rust translation of BRepGraph_Tool::Face and the UV-bounds helpers of the anonymous
// namespace (ModelingData/TKBRep/BRepGraph/BRepGraph_Tool.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Tool::Face`: the face tolerance, surface, outer wire and bounds. The
//! surface adaptors are in `face_adaptors` and re-exported here.
pub use super::face_adaptors::{
    surface_adaptor, surface_adaptor_bounded, surface_adaptor_ref, surface_adaptor_ref_bounded,
    surface_adaptor_usage, surface_adaptor_usage_bounded,
};
use super::{FaceUsage, co_edge, face_of_ref};
use crate::brep_graph::refs_iterator::{CoEdgesOfWire, RefsWireOfFace};
use crate::brep_graph::{BRepGraph, FaceId, FaceRefId, WireId};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::toploc::Location;

/// The UV bounds `(u_min, u_max, v_min, v_max)` of `Bounds`.
pub type UvBounds = (f64, f64, f64, f64);

/// `Usage(theGraph, theFaceRef)`: the active face of an active reference with the
/// reference orientation and the identity location; the invalid usage otherwise.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceRefId, tool};
/// assert!(!tool::face::usage(&BRepGraph::new(), FaceRefId::new(0)).is_valid());
/// ```
pub fn usage(graph: &BRepGraph, face_ref: FaceRefId) -> FaceUsage {
    if !face_ref.is_valid() || graph.refs().generic().is_removed(face_ref.into()) {
        return FaceUsage::default();
    }
    let entry = graph.inc_storage().face_ref(face_ref);
    if !entry.child_face_id.is_valid() || entry.child_face_id.is_removed(graph) {
        return FaceUsage::default();
    }
    FaceUsage {
        def_id: entry.child_face_id,
        location: Location::default(),
        orientation: entry.orientation.orientation(),
    }
}

/// `Tolerance(theGraph, theFace)`.
pub fn tolerance(graph: &BRepGraph, face: FaceId) -> f64 {
    graph.inc_storage().face(face).tolerance
}

/// `Tolerance(theGraph, theFaceRef)`: 0 for an inactive reference or face.
pub fn tolerance_ref(graph: &BRepGraph, face_ref: FaceRefId) -> f64 {
    let face = face_of_ref(graph, face_ref);
    if face.is_valid() {
        tolerance(graph, face)
    } else {
        0.0
    }
}

/// `HasSurface(theGraph, theFace)`: `Topo().Faces().Surface` is not null.
pub fn has_surface(graph: &BRepGraph, face: FaceId) -> bool {
    graph.topo().faces().surface(face).is_some()
}

/// `HasSurface(theGraph, theFaceRef)`.
pub fn has_surface_ref(graph: &BRepGraph, face_ref: FaceRefId) -> bool {
    let face = face_of_ref(graph, face_ref);
    face.is_valid() && has_surface(graph, face)
}

/// `Surface(theGraph, theFace)`: the surface of an active surface use, null otherwise.
pub fn surface(graph: &BRepGraph, face: FaceId) -> Handle<dyn GeomSurface> {
    let storage = graph.inc_storage();
    let rep = storage.face(face).surface_rep_id;
    if !rep.is_valid_for(storage.nb_face_surfaces()) || storage.is_removed(rep) {
        return Handle::default();
    }
    storage.face_surface_rep(rep).surface.clone()
}

/// `Surface(theGraph, theFaceRef)`: null for an inactive reference or face.
pub fn surface_ref(graph: &BRepGraph, face_ref: FaceRefId) -> Handle<dyn GeomSurface> {
    let face = face_of_ref(graph, face_ref);
    if face.is_valid() {
        surface(graph, face)
    } else {
        Handle::default()
    }
}

/// `NbWires(theGraph, theFace)`: the wire references of the face (removed included).
pub fn nb_wires(graph: &BRepGraph, face: FaceId) -> u32 {
    graph.inc_storage().face_relations(face).wire_ref_ids.len() as u32
}

/// `NbWires(theGraph, theFaceRef)`: 0 for an inactive reference or face.
pub fn nb_wires_ref(graph: &BRepGraph, face_ref: FaceRefId) -> u32 {
    let face = face_of_ref(graph, face_ref);
    if face.is_valid() {
        nb_wires(graph, face)
    } else {
        0
    }
}

/// `Bounds(theGraph, theFace, ...)`: the surface bounds, zeros without a surface.
pub fn bounds(graph: &BRepGraph, face: FaceId) -> UvBounds {
    surface(graph, face)
        .get()
        .map_or((0.0, 0.0, 0.0, 0.0), |surface| surface.bounds())
}

/// `Bounds(theGraph, theFaceRef, ...)`: zeros for an inactive reference or face.
pub fn bounds_ref(graph: &BRepGraph, face_ref: FaceRefId) -> UvBounds {
    let face = face_of_ref(graph, face_ref);
    if face.is_valid() {
        bounds(graph, face)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}

/// `OuterWire(theGraph, theFace)`: among the active wires with UV bounds (from their
/// p-curve adaptors, enlarged by Confusion), the last one whose box contains the
/// current candidate's within PConfusion, starting from the first; invalid when no
/// wire has bounds.
pub fn outer_wire(graph: &BRepGraph, face: FaceId) -> Result<WireId, Failure> {
    let mut result: Option<(WireId, UvBounds)> = None;
    for wire_ref in RefsWireOfFace::new(graph, face) {
        let wire = graph.inc_storage().wire_ref(wire_ref).child_wire_id;
        let Some(current) = wire_uv_bounds(graph, wire)? else {
            continue;
        };
        if result.is_none_or(|(_, best)| contains(current, best)) {
            result = Some((wire, current));
        }
    }
    Ok(result.map_or(WireId::default(), |(wire, _)| wire))
}

/// `OuterWire(theGraph, theFaceRef)`: invalid for an inactive reference or face.
pub fn outer_wire_ref(graph: &BRepGraph, face_ref: FaceRefId) -> Result<WireId, Failure> {
    let face = face_of_ref(graph, face_ref);
    if face.is_valid() {
        outer_wire(graph, face)
    } else {
        Ok(WireId::default())
    }
}

// The containment test of `OuterWire`, with its operation order.
fn contains(current: UvBounds, best: UvBounds) -> bool {
    let tolerance = precision::p_confusion();
    (current.0 - best.0) <= tolerance
        && (current.1 - best.1) >= -tolerance
        && (current.2 - best.2) <= tolerance
        && (current.3 - best.3) >= -tolerance
}

// `wireUVBounds` over `addWireUVBounds`: the box of the wire's active coedges'
// initialised p-curve adaptors, `None` when it stays void.
fn wire_uv_bounds(graph: &BRepGraph, wire: WireId) -> Result<Option<UvBounds>, Failure> {
    let mut uv_box = BoundingBox2::default();
    for co_edge in CoEdgesOfWire::new(graph, wire) {
        let adaptor = co_edge::pcurve_adaptor(graph, co_edge)?;
        if adaptor.is_initialized() {
            Add2dCurve::add_adaptor(&adaptor, precision::confusion(), &mut uv_box)?;
        }
    }
    if uv_box.is_void() {
        return Ok(None);
    }
    let [u_min, v_min, u_max, v_max] = uv_box.get()?;
    Ok(Some((u_min, u_max, v_min, v_max)))
}
