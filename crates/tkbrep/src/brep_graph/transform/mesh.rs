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
// Rust translation of transformTriangulation, transformPolygon3D, transformExistingMesh
// and applyMeshCopy of ModelingData/TKBRep/BRepGraph/BRepGraph_Transform.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The mesh steps of a transform: the persistent triangulations, 3D polygons and
//! polygons on triangulation, transformed in place in the copied range or copied (with
//! per-handle deduplication) over the whole graph; the runtime mesh cache is cleared,
//! never transformed.
use super::{CopiedRange, ids_in_range};
use crate::brep_graph::tool;
use crate::brep_graph::*;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::{Transform3, TransformForm};
use rustycad_tkmath::poly::{
    Polygon2D, Polygon3D, PolygonOnTriangulation, Triangle, Triangulation,
};
use std::collections::HashMap;

/// `transformTriangulation`: scales the deflection and transforms the nodes, the UV
/// nodes (through the surface's parametric transformation, when it is not the
/// identity) and the normals in place; a negative transformation flips the winding so
/// the normals stay outward.
fn transform_triangulation(
    triangulation: &Handle<Triangulation>,
    transform: &Transform3,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    let scale = transform.scale_factor().abs();
    triangulation.set_deflection(triangulation.deflection() * scale);
    for index in 1..=triangulation.nb_nodes() {
        let node = triangulation.node(index)?.transformed(*transform);
        triangulation.set_node(index, node)?;
    }
    if triangulation.has_uv_nodes() && !surface.is_null() {
        transform_uv_nodes(triangulation, transform, surface)?;
    }
    if triangulation.has_normals() {
        for index in 1..=triangulation.nb_nodes() {
            let normal = triangulation.normal(index)?.transformed(*transform);
            triangulation.set_normal(index, normal)?;
        }
    }
    if transform.is_negative() {
        flip_winding(triangulation)?;
    }
    Ok(())
}

fn transform_uv_nodes(
    triangulation: &Handle<Triangulation>,
    transform: &Transform3,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    let uv_transform = surface.parametric_transformation(*transform)?;
    if uv_transform.form() == TransformForm::Identity {
        return Ok(());
    }
    for index in 1..=triangulation.nb_nodes() {
        let mut uv = triangulation.uv_node(index)?;
        uv_transform.transforms(uv.change_coord());
        triangulation.set_uv_node(index, uv)?;
    }
    Ok(())
}

fn flip_winding(triangulation: &Handle<Triangulation>) -> Result<(), Failure> {
    for index in 1..=triangulation.nb_triangles() {
        let (n1, n2, n3) = triangulation.triangle(index)?.get();
        triangulation.set_triangle(index, Triangle::new(n1, n3, n2))?;
    }
    Ok(())
}

/// `transformPolygon3D`: scales the deflection and transforms the nodes in place.
fn transform_polygon3d(polygon: &Handle<Polygon3D>, transform: &Transform3) {
    polygon.set_deflection(polygon.deflection() * transform.scale_factor().abs());
    polygon.change_nodes(|nodes| {
        for node in nodes.iter_mut() {
            node.transform(*transform);
        }
    });
}

// Scales the deflection of a polygon on triangulation (its nodes are indices).
fn scale_polygon_on_tri(polygon: &Handle<PolygonOnTriangulation>, transform: &Transform3) {
    polygon.set_deflection(polygon.deflection() * transform.scale_factor().abs());
}

/// `transformExistingMesh`: the persistent mesh already copied into the range is
/// transformed in place when `do_transform` (it is a fresh copy), and the runtime mesh
/// cache of every copied face, edge and coedge is cleared.
pub(crate) fn transform_existing_mesh(
    graph: &mut BRepGraph,
    range: &CopiedRange,
    transform: &Transform3,
    do_transform: bool,
) -> Result<(), Failure> {
    for face in ids_in_range::<FaceTag>(graph, range) {
        let triangulation = graph.mesh().persistent().faces().triangulation(face);
        if !triangulation.is_null() && do_transform {
            let surface = tool::face::surface(graph, face);
            transform_triangulation(&triangulation, transform, &surface)?;
        }
        graph.mesh().editor().faces().clear(face)?;
    }
    for edge in ids_in_range::<EdgeTag>(graph, range) {
        let polygon = graph.mesh().persistent().edges().polygon3d(edge);
        if !polygon.is_null() && do_transform {
            transform_polygon3d(&polygon, transform);
        }
        graph.mesh().editor().edges().clear(edge);
    }
    for co_edge in ids_in_range::<CoEdgeTag>(graph, range) {
        let polygon = graph
            .mesh()
            .persistent()
            .co_edges()
            .polygon_on_triangulation(co_edge);
        if !polygon.is_null() && do_transform {
            scale_polygon_on_tri(&polygon, transform);
        }
        graph.mesh().editor().co_edges().clear(co_edge);
    }
    Ok(())
}

// The per-handle deduplication maps of `applyMeshCopy`: a source handle shared by
// several nodes maps to one copy. Keyed by object address, as the native handle maps;
// each entry keeps its source handle alive (the native key is a handle), so a freed
// source address cannot be reused by a later allocation.
type CopyMap<T> = HashMap<usize, (Handle<T>, Handle<T>)>;

#[derive(Default)]
struct MeshCopies {
    triangulations: CopyMap<Triangulation>,
    polygons3d: CopyMap<Polygon3D>,
    polygons2d: CopyMap<Polygon2D>,
    polygons_on_tri: CopyMap<PolygonOnTriangulation>,
}

fn address<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> usize {
    handle.object_address() as usize
}

/// `applyMeshCopy(theSource, theDest, theTrsf, theDoTransform)`: every persistent
/// mesh representation is replaced by a copy (transformed when `do_transform`) and
/// the runtime mesh cache is cleared. Every native caller passes the transformed graph
/// as both the source and the destination, and no poly source, so the Rust function
/// takes the one graph.
pub(crate) fn copy_mesh(
    graph: &mut BRepGraph,
    transform: &Transform3,
    do_transform: bool,
) -> Result<(), Failure> {
    let mut copies = MeshCopies::default();
    copy_face_meshes(graph, &mut copies, transform, do_transform)?;
    copy_edge_meshes(graph, &mut copies, transform, do_transform)?;
    copy_co_edge_meshes(graph, &mut copies, transform, do_transform)
}

fn copy_face_meshes(
    graph: &mut BRepGraph,
    copies: &mut MeshCopies,
    transform: &Transform3,
    do_transform: bool,
) -> Result<(), Failure> {
    for face in ids_in_range::<FaceTag>(graph, &CopiedRange::default()) {
        graph.mesh().editor().faces().clear(face)?;
        let source = graph.mesh().persistent().faces().triangulation(face);
        let copy = copy_triangulation(graph, face, copies, &source, transform, do_transform)?;
        if !copy.is_null() {
            graph
                .editor()
                .faces()
                .set_persistent_triangulation(face, &copy)?;
        }
    }
    Ok(())
}

// `copyOneTri`: the deduplicated copy, transformed on the face's (already
// transformed) surface.
fn copy_triangulation(
    graph: &BRepGraph,
    face: FaceId,
    copies: &mut MeshCopies,
    source: &Handle<Triangulation>,
    transform: &Transform3,
    do_transform: bool,
) -> Result<Handle<Triangulation>, Failure> {
    if source.is_null() {
        return Ok(Handle::default());
    }
    if let Some((_, existing)) = copies.triangulations.get(&address(source)) {
        return Ok(existing.clone());
    }
    let copy = source.copy();
    if do_transform {
        transform_triangulation(&copy, transform, &tool::face::surface(graph, face))?;
    }
    copies
        .triangulations
        .insert(address(source), (source.clone(), copy.clone()));
    Ok(copy)
}

fn copy_edge_meshes(
    graph: &mut BRepGraph,
    copies: &mut MeshCopies,
    transform: &Transform3,
    do_transform: bool,
) -> Result<(), Failure> {
    for edge in ids_in_range::<EdgeTag>(graph, &CopiedRange::default()) {
        let source = graph.mesh().persistent().edges().polygon3d(edge);
        if !source.is_null() {
            let copy = copy_polygon3d(copies, &source, transform, do_transform)?;
            graph
                .editor()
                .edges()
                .set_persistent_polygon3d(edge, &copy)?;
        }
        graph.mesh().editor().edges().clear(edge);
    }
    Ok(())
}

// `copyOnePolygon3D` (a non-null source only).
fn copy_polygon3d(
    copies: &mut MeshCopies,
    source: &Handle<Polygon3D>,
    transform: &Transform3,
    do_transform: bool,
) -> Result<Handle<Polygon3D>, Failure> {
    if let Some((_, existing)) = copies.polygons3d.get(&address(source)) {
        return Ok(existing.clone());
    }
    let copy = source.copy()?;
    if do_transform {
        transform_polygon3d(&copy, transform);
    }
    copies
        .polygons3d
        .insert(address(source), (source.clone(), copy.clone()));
    Ok(copy)
}

fn copy_co_edge_meshes(
    graph: &mut BRepGraph,
    copies: &mut MeshCopies,
    transform: &Transform3,
    do_transform: bool,
) -> Result<(), Failure> {
    for co_edge in ids_in_range::<CoEdgeTag>(graph, &CopiedRange::default()) {
        let persistent = graph.mesh().persistent().co_edges();
        let polygon2d = persistent.polygon_on_surface(co_edge);
        let on_tri = persistent.polygon_on_triangulation(co_edge);
        if !polygon2d.is_null() {
            let copy = copy_polygon2d(copies, &polygon2d);
            graph
                .editor()
                .co_edges()
                .set_persistent_polygon2d(co_edge, &copy)?;
        }
        if !on_tri.is_null() {
            let copy = copy_polygon_on_tri(copies, &on_tri, transform, do_transform)?;
            graph
                .editor()
                .co_edges()
                .set_persistent_polygon_on_tri(co_edge, &copy)?;
        }
        graph.mesh().editor().co_edges().clear(co_edge);
    }
    Ok(())
}

// `copyOnePolygon2D` (a non-null source only): UV polygons are not transformed.
fn copy_polygon2d(copies: &mut MeshCopies, source: &Handle<Polygon2D>) -> Handle<Polygon2D> {
    copies
        .polygons2d
        .entry(address(source))
        .or_insert_with(|| (source.clone(), source.copy()))
        .1
        .clone()
}

// `copyOnePolygonOnTri` (a non-null source only).
fn copy_polygon_on_tri(
    copies: &mut MeshCopies,
    source: &Handle<PolygonOnTriangulation>,
    transform: &Transform3,
    do_transform: bool,
) -> Result<Handle<PolygonOnTriangulation>, Failure> {
    if let Some((_, existing)) = copies.polygons_on_tri.get(&address(source)) {
        return Ok(existing.clone());
    }
    let copy = source.copy()?;
    if do_transform {
        scale_polygon_on_tri(&copy, transform);
    }
    copies
        .polygons_on_tri
        .insert(address(source), (source.clone(), copy.clone()));
    Ok(copy)
}
