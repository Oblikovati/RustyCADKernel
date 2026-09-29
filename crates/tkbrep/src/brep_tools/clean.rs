// Created on: 1993-01-21
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of BRepTools::Clean, CleanGeometry and RemoveUnusedPCurves
// (ModelingData/TKBRep/BRepTools/BRepTools.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Removing meshes and geometry from a shape.
use crate::brep::access::edge_tshape;
use crate::brep::{BRepBuilder, BRepCurveRepresentation, BRepTool};
use crate::topexp::{self, Explorer};
use crate::topods::{self, Edge, Face, Shape, TShape};
use crate::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::poly::{PolygonOnTriangulation, Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;

/// `BRepTools::Clean(theShape, theForce)`: removes the triangulation of each geometric
/// face (with the polygons on it of its edges), then the 3D polygon of each geometric
/// edge; with `force`, also every polygon on triangulation of the edges. Faces and
/// edges are processed once per TShape and orientation-free identity, whatever their
/// location; `clean(&shape, false)?`.
pub fn clean(shape: &Shape, force: bool) -> Result<(), Failure> {
    if shape.is_null() {
        return Ok(());
    }
    let mut processed = MapOfShape::default();
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        if processed.add(face.located(&Location::default(), false)?) {
            clean_face(&topods::face(&face)?)?;
        }
    }
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&edge.located(&Location::default(), false)?)?;
        if processed.add(edge.clone().into_shape()) {
            clean_edge(&edge, force)?;
        }
    }
    Ok(())
}

/// A geometric face loses its triangulation and its edges the polygons on it.
fn clean_face(face: &Face) -> Result<(), Failure> {
    if !BRepTool::is_geometric_face(face)? {
        // Without a surface the triangulation cannot be recomputed: keep it.
        return Ok(());
    }
    let (triangulation, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
    if triangulation.is_null() {
        return Ok(());
    }
    let builder = BRepBuilder::default();
    let no_polygon = Handle::<PolygonOnTriangulation>::default();
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&edge)?;
        builder.update_edge_polygon_on_triangulation(
            &edge,
            &no_polygon,
            &triangulation,
            &location,
        )?;
    }
    builder.update_face_triangulation(face, &Handle::<Triangulation>::default(), true)
}

/// A geometric edge (at the identity location) loses its 3D polygon and, with
/// `force`, its polygons on triangulation.
fn clean_edge(edge: &Edge, force: bool) -> Result<(), Failure> {
    if !BRepTool::is_geometric_edge(edge)? {
        // Without a curve the polygon cannot be recomputed: keep it.
        return Ok(());
    }
    let builder = BRepBuilder::default();
    let (polygon, _) = BRepTool::polygon3d(edge)?;
    if !polygon.is_null() {
        builder.update_edge_polygon3d(edge, &Handle::default(), &Location::default())?;
    }
    if force {
        let tedge = edge_tshape(edge, "BRepTools::Clean")?;
        tedge.change_curves(|curves| {
            curves.retain(|r| r.get().is_none_or(|r| !r.is_polygon_on_triangulation()));
        });
        tedge.set_modified(true);
    }
    Ok(())
}

/// `BRepTools::CleanGeometry(theShape)`: removes the p-curves of the edges on each face,
/// the surface of each face (keeping its location and tolerance) and the 3D curve of
/// each edge, then the unused p-curves; `clean_geometry(&shape)?`.
pub fn clean_geometry(shape: &Shape) -> Result<(), Failure> {
    if shape.is_null() {
        return Ok(());
    }
    let builder = BRepBuilder::default();
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = topods::face(&face)?;
        let (surface, location) = BRepTool::surface_with_location(&face)?;
        for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = topods::edge(&edge)?;
            let tolerance = BRepTool::edge_tolerance(&edge)?;
            let no_pcurve = Handle::<dyn Geom2dCurve>::default();
            builder.update_edge_pcurve(&edge, &no_pcurve, &surface, &location, tolerance, None)?;
        }
        let no_surface = Handle::<dyn GeomSurface>::default();
        let tolerance = BRepTool::face_tolerance(&face)?;
        builder.update_face(&face, &no_surface, &face.location().clone(), tolerance)?;
    }
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&edge)?;
        let tolerance = BRepTool::edge_tolerance(&edge)?;
        let no_curve = Handle::<dyn GeomCurve>::default();
        builder.update_edge_located_curve(&edge, &no_curve, &Location::default(), tolerance)?;
    }
    remove_unused_pcurves(shape)
}

/// `BRepTools::RemoveUnusedPCurves(S)`: removes from each edge the curves on a surface,
/// and the regularities between surfaces, that no face of `shape` uses (surfaces are
/// compared by object identity); `remove_unused_pcurves(&shape)?`.
pub fn remove_unused_pcurves(shape: &Shape) -> Result<(), Failure> {
    let mut used: Vec<Handle<dyn GeomSurface>> = Vec::new();
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        used.push(BRepTool::surface_with_location(&topods::face(&face)?)?.0);
    }
    let mut edges = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, ShapeEnum::Edge, &mut edges)?;
    for index in 1..=edges.extent() {
        let edge = edges.find_key(index)?.clone();
        let tedge = edge_tshape(&edge, "BRepTools::RemoveUnusedPCurves")?;
        let mut kept = Vec::new();
        for representation in tedge.curves() {
            if !is_unused(&representation, &used)? {
                kept.push(representation);
            }
        }
        tedge.change_curves(|curves| *curves = kept);
    }
    Ok(())
}

/// Whether a curve on a surface, or a regularity, refers to a surface that no face uses.
fn is_unused(
    representation: &Handle<dyn BRepCurveRepresentation>,
    used: &[Handle<dyn GeomSurface>],
) -> Result<bool, Failure> {
    let Some(stored) = representation.get() else {
        return Ok(false);
    };
    let is_used = |surface: &Handle<dyn GeomSurface>| used.iter().any(|s| s == surface);
    if stored.is_curve_on_surface() {
        return Ok(!is_used(&stored.surface()?));
    }
    if stored.is_regularity() {
        return Ok(!is_used(&stored.surface()?) || !is_used(&stored.surface2()?));
    }
    Ok(false)
}
