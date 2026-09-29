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
// Rust translation of appendVertexRef, registerOrReuseVertex (both), appendEdgeVertexRef,
// registerEdge, appendWireDef, appendFaceCoEdge, appendWireCoEdge and registerFaceData of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Populate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The registration of extracted data into the storage, with TShape deduplication.
use super::context::{BuildContext, definition_shape_key};
use super::extract::{ExtractedEdge, ExtractedVertex, raw_vertex_point};
use super::face::FaceBuildData;
use super::locations::located_point;
use super::supplement::attach_supplement;
use crate::brep::BRepTool;
use crate::brep_graph::{EdgeId, FaceId, NodeKind, VertexId, VertexRefId, WireId};
use crate::brep_graph_inc::{ParityOrientation, Storage};
use crate::topods::{Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

/// `appendVertexRef`: a vertex reference owned by the edge.
pub(super) fn append_vertex_ref(
    storage: &mut Storage,
    vertex: VertexId,
    orientation: ParityOrientation,
    parent_edge: EdgeId,
) -> VertexRefId {
    let reference = storage.append_vertex_ref();
    let entry = storage.change_vertex_ref(reference);
    entry.child_vertex_id = vertex;
    entry.parent_edge_id = parent_edge;
    entry.orientation = orientation;
    reference
}

/// `registerOrReuseVertex(theBuild, theVertex, thePoint, theTolerance,
/// theDefinitionLocation, theIsGenerated)`: the existing vertex of the TShape at the
/// location, else a new one (bound and with an original unless generated).
fn register_or_reuse_vertex_at(
    build: &mut BuildContext<'_>,
    vertex: &Vertex,
    point: Point3,
    tolerance: f64,
    definition_location: &Location,
    is_generated: bool,
) -> Result<VertexId, Failure> {
    if vertex.is_null() {
        return Ok(VertexId::default());
    }
    let existing = build.find_existing_node(vertex, NodeKind::VERTEX, definition_location)?;
    if existing.is_valid() {
        return Ok(VertexId::from_node_id(existing));
    }
    let id = build.storage.append_vertex();
    let def = build.storage.change_vertex(id);
    def.point = point;
    def.tolerance = tolerance;
    if !is_generated {
        build.bind_located_node(vertex, id.into(), definition_location, true)?;
        let key = definition_shape_key(vertex, definition_location)?;
        build.storage.bind_original(id.into(), &key);
    }
    Ok(id)
}

/// `registerOrReuseVertex(theBuild, theVertex, theDefinitionLocation)`: a standalone
/// vertex with its raw point moved to the definition frame; invalid without one.
pub(super) fn register_or_reuse_vertex(
    build: &mut BuildContext<'_>,
    vertex: &Vertex,
    definition_location: &Location,
) -> Result<VertexId, Failure> {
    if vertex.is_null() {
        return Ok(VertexId::default());
    }
    let Some(point) = raw_vertex_point(vertex) else {
        return Ok(VertexId::default());
    };
    let point = located_point(point, definition_location);
    let tolerance = BRepTool::vertex_tolerance(vertex)?;
    register_or_reuse_vertex_at(build, vertex, point, tolerance, definition_location, false)
}

// `appendEdgeVertexRef`: the reference to the (reused) vertex of an extracted boundary.
fn append_edge_vertex_ref(
    build: &mut BuildContext<'_>,
    vertex: &ExtractedVertex,
    orientation: ParityOrientation,
    parent_edge: EdgeId,
) -> Result<VertexRefId, Failure> {
    if vertex.shape.is_null() {
        return Ok(VertexRefId::default());
    }
    let id = register_or_reuse_vertex_at(
        build,
        &vertex.shape,
        vertex.point,
        vertex.tolerance,
        &vertex.definition_location,
        vertex.is_generated,
    )?;
    Ok(append_vertex_ref(
        build.storage,
        id,
        orientation,
        parent_edge,
    ))
}

/// `registerEdge`: the existing edge of a non-generated TShape at the location (which
/// gains a 3D polygon use when it had none), else a new edge with its curve use, its
/// vertex references, the vertex-edge relations and its polygon use.
pub(super) fn register_edge(
    build: &mut BuildContext<'_>,
    data: &ExtractedEdge,
) -> Result<EdgeId, Failure> {
    if !data.is_generated {
        let existing =
            build.find_existing_node(&data.shape, NodeKind::EDGE, &data.definition_location)?;
        if existing.is_valid() {
            let edge = EdgeId::from_node_id(existing);
            complete_existing_edge(build.storage, edge, data);
            return Ok(edge);
        }
    }
    let edge = build.storage.append_edge();
    build.storage.change_edge(edge).tolerance = data.tolerance;
    if data.curve3d.is_some() {
        let rep = build.storage.append_edge_curve3d_rep();
        let record = build.storage.change_edge_curve3d_rep(rep);
        record.curve = data.curve3d.clone();
        record.parent_edge_id = edge;
        record.param_first = data.param_first;
        record.param_last = data.param_last;
        build.storage.change_edge(edge).curve3d_rep_id = rep;
    }
    link_edge_vertices(build, edge, data)?;
    append_edge_polygon(build.storage, edge, data);
    if !data.is_generated {
        bind_edge(build, edge, data)?;
    }
    Ok(edge)
}

// The polygon of a reused edge that had no polygon use.
fn complete_existing_edge(storage: &mut Storage, edge: EdgeId, data: &ExtractedEdge) {
    if !storage.edge(edge).polygon3d_rep_id.is_valid() {
        append_edge_polygon(storage, edge, data);
    }
}

fn append_edge_polygon(storage: &mut Storage, edge: EdgeId, data: &ExtractedEdge) {
    if data.polygon3d.is_null() {
        return;
    }
    let rep = storage.append_edge_polygon3d_rep();
    let record = storage.change_edge_polygon3d_rep(rep);
    record.polygon = data.polygon3d.clone();
    record.parent_edge_id = edge;
    storage.change_edge(edge).polygon3d_rep_id = rep;
}

// The start (FORWARD) and end (REVERSED) references and the vertex-edge relations.
fn link_edge_vertices(
    build: &mut BuildContext<'_>,
    edge: EdgeId,
    data: &ExtractedEdge,
) -> Result<(), Failure> {
    let start =
        append_edge_vertex_ref(build, &data.start_vertex, ParityOrientation::FORWARD, edge)?;
    build.storage.change_edge(edge).start_vertex_ref_id = start;
    let end = append_edge_vertex_ref(build, &data.end_vertex, ParityOrientation::REVERSED, edge)?;
    build.storage.change_edge(edge).end_vertex_ref_id = end;
    for reference in [start, end] {
        if !reference.is_valid() {
            continue;
        }
        let vertex = build.storage.vertex_ref(reference).child_vertex_id;
        if vertex.is_valid() {
            build.storage.attach_edge_to_vertex(edge, vertex);
        }
    }
    Ok(())
}

// The bindings, the original and the supplement of a new non-generated edge.
fn bind_edge(
    build: &mut BuildContext<'_>,
    edge: EdgeId,
    data: &ExtractedEdge,
) -> Result<(), Failure> {
    build.bind_located_node(&data.shape, edge.into(), &data.definition_location, true)?;
    let key = definition_shape_key(&data.shape, &data.definition_location)?;
    build.storage.bind_original(edge.into(), &key);
    attach_supplement(build.supplement, edge.into(), &data.shape)
}

/// `appendWireDef`: a new wire, bound with its original unless generated.
pub(super) fn append_wire_def(
    build: &mut BuildContext<'_>,
    wire: &Wire,
    definition_location: &Location,
    is_generated: bool,
) -> Result<WireId, Failure> {
    let id = build.storage.append_wire();
    if !is_generated {
        build.bind_located_node(wire, id.into(), definition_location, true)?;
        let key = definition_shape_key(wire, definition_location)?;
        build.storage.bind_original(id.into(), &key);
    }
    Ok(id)
}

/// `appendFaceCoEdge`: the coedge of an edge use in a face wire, with its polygon on
/// surface, p-curve and polygon on triangulation uses. An INTERNAL or EXTERNAL edge
/// raises Standard_ProgramError through the parity orientation.
fn append_face_co_edge(
    storage: &mut Storage,
    wire: WireId,
    face: FaceId,
    data: &ExtractedEdge,
    edge: EdgeId,
) -> Result<(), Failure> {
    let orientation = ParityOrientation::new(data.orientation_in_wire)?;
    let co_edge = storage.create_co_edge_use(wire, edge, face, orientation);
    if data.poly_on_surf.is_some() {
        let rep = storage.append_co_edge_polygon2d_rep();
        let record = storage.change_co_edge_polygon2d_rep(rep);
        record.polygon = data.poly_on_surf.clone();
        record.parent_co_edge_id = co_edge;
        storage.change_co_edge(co_edge).polygon2d_rep_id = rep;
    }
    if data.pcurve2d.is_some() {
        let rep = storage.append_co_edge_curve2d_rep();
        let record = storage.change_co_edge_curve2d_rep(rep);
        record.curve = data.pcurve2d.clone();
        record.parent_co_edge_id = co_edge;
        record.param_first = data.pc_first;
        record.param_last = data.pc_last;
        storage.change_co_edge(co_edge).curve2d_rep_id = rep;
    }
    if data.poly_on_tri.is_some() {
        let rep = storage.append_co_edge_polygon_on_tri_rep();
        let record = storage.change_co_edge_polygon_on_tri_rep(rep);
        record.polygon = data.poly_on_tri.clone();
        record.parent_co_edge_id = co_edge;
        storage.change_co_edge(co_edge).polygon_on_tri_rep_id = rep;
    }
    Ok(())
}

/// `appendWireCoEdge`: the coedge of an edge use in a free wire (no face).
pub(super) fn append_wire_co_edge(
    storage: &mut Storage,
    wire: WireId,
    edge: EdgeId,
    orientation: Orientation,
) -> Result<(), Failure> {
    let parity = ParityOrientation::new(orientation)?;
    storage.create_co_edge_use(wire, edge, FaceId::default(), parity);
    Ok(())
}

/// `registerFaceData`: the surface and triangulation uses of every queued face, then
/// its wires, edges and coedges in order, each wire canonicalised.
pub(super) fn register_face_data(
    build: &mut BuildContext<'_>,
    faces: &[FaceBuildData],
) -> Result<(), Failure> {
    for data in faces {
        register_face_geometry(build.storage, data);
        for wire_data in &data.wires {
            let wire = append_wire_def(
                build,
                &wire_data.shape,
                &wire_data.definition_location,
                wire_data.is_generated,
            )?;
            let orientation = ParityOrientation::new(wire_data.shape.orientation())?;
            build
                .storage
                .attach_wire_to_face(data.face_id, wire, orientation);
            for edge_data in &wire_data.edges {
                let edge = register_edge(build, edge_data)?;
                append_face_co_edge(build.storage, wire, data.face_id, edge_data, edge)?;
            }
            build.has_wire_order_warnings =
                build.has_wire_order_warnings || wire_data.has_partial_explorer_order;
            let status = build.storage.canonicalize_wire_co_edge_order_status(wire);
            build.record_wire_order_status(status);
        }
    }
    Ok(())
}

// The tolerance, surface use and triangulation use of one face.
fn register_face_geometry(storage: &mut Storage, data: &FaceBuildData) {
    let face = data.face_id;
    storage.change_face(face).tolerance = data.tolerance;
    if data.surface.is_some() {
        let rep = storage.append_face_surface_rep();
        let record = storage.change_face_surface_rep(rep);
        record.surface = data.surface.clone();
        record.parent_face_id = face;
        storage.change_face(face).surface_rep_id = rep;
    }
    if data.active_triangulation.is_some() {
        let rep = storage.append_face_triangulation_rep();
        let record = storage.change_face_triangulation_rep(rep);
        record.triangulation = data.active_triangulation.clone();
        record.parent_face_id = face;
        storage.change_face(face).triangulation_rep_id = rep;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_wire_coedges_refuse_internal_edges() {
        let mut storage = Storage::new();
        let (wire, edge) = (storage.append_wire(), storage.append_edge());
        append_wire_co_edge(&mut storage, wire, edge, Orientation::Reversed).unwrap();
        assert_eq!(storage.nb_co_edges(), 1);
        let error =
            append_wire_co_edge(&mut storage, wire, edge, Orientation::Internal).unwrap_err();
        assert_eq!(error.exception_type(), "Standard_ProgramError");
        let reference = append_vertex_ref(
            &mut storage,
            VertexId::new(2),
            ParityOrientation::REVERSED,
            edge,
        );
        assert_eq!(storage.vertex_ref(reference).parent_edge_id, edge);
    }
}
