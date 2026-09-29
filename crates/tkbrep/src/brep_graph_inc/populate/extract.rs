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
// Rust translation of the extraction structs and functions (ExtractedVertex,
// ExtractedEdge, ExtractedWire, FaceBuildData, findStoredPCurve, extractStoredPCurve,
// rawVertexPoint, edgeVertices, extractEdgeDefinition, extractEdgeInFace) of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Populate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The per-edge extraction of population: definition-frame geometry, vertices, the
//! stored p-curve and the polygons of one edge use.
use super::locations::{located_curve, located_point, located_polygon3d};
use crate::brep::{BRepTEdge, BRepTVertex, BRepTool};
use crate::topods::{self, Edge, Face, ShapeIterator, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;

/// `ExtractedVertex`: a vertex use with its definition-frame point.
#[derive(Clone, Default)]
pub(super) struct ExtractedVertex {
    pub(super) shape: Vertex,
    pub(super) point: Point3,
    pub(super) tolerance: f64,
    pub(super) definition_location: Location,
    pub(super) is_generated: bool,
}

/// `ExtractedEdge`: an edge use with its definition-frame geometry and, inside a face,
/// its p-curve and polygons on the face.
#[derive(Clone)]
pub(super) struct ExtractedEdge {
    pub(super) shape: Edge,
    pub(super) curve3d: Handle<dyn GeomCurve>,
    pub(super) param_first: f64,
    pub(super) param_last: f64,
    pub(super) tolerance: f64,
    pub(super) start_vertex: ExtractedVertex,
    pub(super) end_vertex: ExtractedVertex,
    pub(super) orientation_in_wire: Orientation,
    pub(super) pcurve2d: Handle<dyn Geom2dCurve>,
    pub(super) pc_first: f64,
    pub(super) pc_last: f64,
    pub(super) polygon3d: Handle<Polygon3D>,
    pub(super) poly_on_surf: Handle<Polygon2D>,
    pub(super) poly_on_tri: Handle<PolygonOnTriangulation>,
    pub(super) definition_location: Location,
    pub(super) is_generated: bool,
}

impl Default for ExtractedEdge {
    fn default() -> Self {
        Self {
            shape: Edge::default(),
            curve3d: Handle::default(),
            param_first: 0.0,
            param_last: 0.0,
            tolerance: 0.0,
            start_vertex: ExtractedVertex::default(),
            end_vertex: ExtractedVertex::default(),
            orientation_in_wire: Orientation::Forward,
            pcurve2d: Handle::default(),
            pc_first: 0.0,
            pc_last: 0.0,
            polygon3d: Handle::default(),
            poly_on_surf: Handle::default(),
            poly_on_tri: Handle::default(),
            definition_location: Location::default(),
            is_generated: false,
        }
    }
}

/// `FacePCurveContext`: the stored surface and locations of the face the p-curves
/// are looked up on.
pub(super) struct FacePCurveContext {
    pub(super) raw_surface: Handle<dyn GeomSurface>,
    pub(super) surface_location: Location,
    pub(super) triangulation_location: Location,
}

// A stored p-curve with its parameter range.
type StoredPCurve = (Handle<dyn Geom2dCurve>, f64, f64);

// `findStoredPCurve`: the first representation on (surface, location); a non-parametric
// one ends the search; the second p-curve of a seam serves a reversed edge.
fn find_stored_pcurve(
    tedge: &BRepTEdge,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
    reversed: bool,
) -> Result<Option<StoredPCurve>, Failure> {
    for representation in tedge.curves() {
        let Some(curve) = representation.get() else {
            continue;
        };
        if !curve.is_curve_on_surface_at(surface, location) {
            continue;
        }
        let gcurve = crate::brep::update_curves::as_gcurve(&representation);
        let Some(parametric) = gcurve.get() else {
            return Ok(None);
        };
        let (first, last) = parametric.range();
        let pcurve = if parametric.is_curve_on_closed_surface() && reversed {
            parametric.pcurve2()?
        } else {
            parametric.pcurve()?
        };
        return Ok(Some((pcurve, first, last)));
    }
    Ok(None)
}

// `extractStoredPCurve`: the p-curve stored in the TEdge for the face's raw surface at
// the surface location relative to the edge location (never a computed one).
fn extract_stored_pcurve(
    edge: &Edge,
    context: &FacePCurveContext,
) -> Result<Option<StoredPCurve>, Failure> {
    if context.raw_surface.is_null() {
        return Ok(None);
    }
    let tedge = edge.tshape().cast::<BRepTEdge>();
    let Some(tedge) = tedge.get() else {
        return Ok(None);
    };
    let reversed = edge.orientation() == Orientation::Reversed;
    let expected = context.surface_location.predivided(edge.location())?;
    find_stored_pcurve(tedge, &context.raw_surface, &expected, reversed)
}

/// `rawVertexPoint`: the stored `BRep_TVertex` point, without any location.
pub(super) fn raw_vertex_point(vertex: &Vertex) -> Option<Point3> {
    let tvertex = vertex.tshape().cast::<BRepTVertex>();
    tvertex.get().map(BRepTVertex::pnt)
}

// `edgeVertices`: the last FORWARD and the last REVERSED vertex child of the edge.
fn edge_vertices(edge: &Edge) -> (Vertex, Vertex) {
    let (mut first, mut last) = (Vertex::default(), Vertex::default());
    for child in ShapeIterator::new(edge, false, false) {
        if child.shape_type().ok() != Some(ShapeEnum::Vertex) {
            continue;
        }
        let vertex = topods::Vertex::unchecked(child);
        match vertex.orientation() {
            Orientation::Forward => first = vertex,
            Orientation::Reversed => last = vertex,
            _ => {}
        }
    }
    (first, last)
}

// One boundary vertex of `extractEdgeDefinition`: its point with the edge definition
// location composed with the vertex location.
fn extract_vertex(vertex: &Vertex, edge_location: &Location) -> Result<ExtractedVertex, Failure> {
    let mut extracted = ExtractedVertex::default();
    let Some(point) = (!vertex.is_null())
        .then(|| raw_vertex_point(vertex))
        .flatten()
    else {
        return Ok(extracted);
    };
    extracted.shape = vertex.clone();
    extracted.definition_location = (edge_location * vertex.location())?;
    extracted.point = located_point(point, &extracted.definition_location);
    extracted.tolerance = BRepTool::vertex_tolerance(vertex)?;
    Ok(extracted)
}

/// `extractEdgeDefinition(theEdgeData, theEdge, theParentLocation)`: the tolerance,
/// the definition location, the 3D curve and polygon in definition frame and the
/// boundary vertices of the FORWARD edge.
pub(super) fn extract_edge_definition(
    edge: &Edge,
    parent_location: &Location,
) -> Result<ExtractedEdge, Failure> {
    let mut data = ExtractedEdge {
        shape: edge.clone(),
        tolerance: BRepTool::edge_tolerance(edge)?,
        orientation_in_wire: edge.orientation(),
        definition_location: (parent_location * edge.location())?,
        ..ExtractedEdge::default()
    };
    let curve = BRepTool::curve_with_location(edge)?;
    data.param_first = curve.first;
    data.param_last = curve.last;
    data.curve3d = located_curve(&curve.curve, &(parent_location * &curve.location)?)?;
    let forward = topods::Edge::unchecked(edge.oriented(Orientation::Forward));
    let (first, last) = edge_vertices(&forward);
    data.start_vertex = extract_vertex(&first, &data.definition_location)?;
    data.end_vertex = extract_vertex(&last, &data.definition_location)?;
    let (polygon, polygon_location) = BRepTool::polygon3d(edge)?;
    data.polygon3d = located_polygon3d(&polygon, &(parent_location * &polygon_location)?)?;
    Ok(data)
}

/// `extractEdgeInFace`: the definition, then the stored p-curve on the face, the
/// polygon on the FORWARD face and the polygon on the face's raw triangulation.
pub(super) fn extract_edge_in_face(
    edge: &Edge,
    forward_face: &Face,
    triangulation: &Handle<Triangulation>,
    context: &FacePCurveContext,
    parent_location: &Location,
) -> Result<ExtractedEdge, Failure> {
    let mut data = extract_edge_definition(edge, parent_location)?;
    if let Some((pcurve, first, last)) = extract_stored_pcurve(edge, context)? {
        data.pcurve2d = pcurve;
        data.pc_first = first;
        data.pc_last = last;
    }
    data.poly_on_surf = BRepTool::polygon_on_face(edge, forward_face)?;
    if triangulation.is_some() {
        // The lookup key is the source representation's location (BRep_Tool compares
        // L.Predivided(E.Location()) with the stored one), so pass the raw
        // triangulation location, not the graph placement.
        data.poly_on_tri = BRepTool::polygon_on_triangulation(
            edge,
            triangulation,
            &context.triangulation_location,
        )?;
    }
    Ok(data)
}
