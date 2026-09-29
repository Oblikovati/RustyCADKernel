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

//! The in-graph transform steps on a small editor-built graph: two vertices, a line
//! edge, two planar faces sharing one triangulation, and a part product placing the
//! first face.
use super::*;
use crate::brep_graph::tool;
use crate::brep_graph::{EdgeId, FaceId, NodeId, ProductId, VertexId, WireId};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Plane3, Point3, Vector3};
use rustycad_tkmath::poly::{Polygon3D, Triangle, Triangulation};

struct Fixture {
    graph: BRepGraph,
    edge: EdgeId,
    faces: [FaceId; 2],
    product: ProductId,
    triangulation: Handle<Triangulation>,
}

fn plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::from_pln(Plane3::default())).cast()
}

// One triangle over three nodes of the XY plane, with UV nodes.
fn triangle_mesh() -> Handle<Triangulation> {
    let triangulation = Handle::new(Triangulation::with_sizes(3, 1, true, false).unwrap());
    let nodes = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)];
    for (index, (x, y)) in (1..).zip(nodes) {
        triangulation
            .set_node(index, Point3::new(x, y, 0.0))
            .unwrap();
        let uv = rustycad_tkmath::gp::Point2::new(x, y);
        triangulation.set_uv_node(index, uv).unwrap();
    }
    triangulation
        .set_triangle(1, Triangle::new(1, 2, 3))
        .unwrap();
    triangulation.set_deflection(0.5);
    triangulation
}

fn fixture() -> Fixture {
    let mut graph = BRepGraph::new();
    let v0 = graph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1e-7);
    let v1 = graph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 1e-7);
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).into_cast();
    let edge = graph.editor().edges().add(v0, v1, &line, 0.0, 1.0, 1e-7);
    let face_a = graph
        .editor()
        .faces()
        .add(&plane(), WireId::default(), &[], 1e-7);
    let face_b = graph
        .editor()
        .faces()
        .add(&plane(), WireId::default(), &[], 1e-7);
    let triangulation = triangle_mesh();
    for face in [face_a, face_b] {
        let mut faces = graph.editor().faces();
        faces
            .set_persistent_triangulation(face, &triangulation)
            .unwrap();
    }
    let product = graph
        .editor()
        .products()
        .add(face_a.into(), &Location::default());
    Fixture {
        graph,
        edge,
        faces: [face_a, face_b],
        product,
        triangulation,
    }
}

fn translation() -> Transform3 {
    Transform3::translation_vector(Vector3::new(1.0, 2.0, 3.0))
}

fn mirror() -> Transform3 {
    let mut transform = Transform3::default();
    transform.set_mirror_plane(Axis2::default());
    transform
}

fn point_of(graph: &BRepGraph, vertex: u32) -> Point3 {
    graph
        .topo()
        .vertices()
        .definition(VertexId::new(vertex))
        .point
}

#[test]
fn a_geometry_transform_moves_points_and_replaces_geometry_handles() {
    let mut f = fixture();
    let old_surface = tool::face::surface(&f.graph, f.faces[0]);
    let old_curve = tool::edge::curve(&f.graph, f.edge);
    transform_whole_graph(&mut f.graph, &translation(), true, false).unwrap();
    assert_eq!(point_of(&f.graph, 1), Point3::new(2.0, 2.0, 3.0));
    let surface = tool::face::surface(&f.graph, f.faces[0]);
    assert!(surface != old_surface);
    assert!(tool::edge::curve(&f.graph, f.edge) != old_curve);
    assert_eq!(tool::edge::range(&f.graph, f.edge), (0.0, 1.0));
    // The shared source surface is not mutated.
    let origin = old_surface.value(0.0, 0.0).unwrap();
    assert_eq!(origin, Point3::new(0.0, 0.0, 0.0));
    // Without a mesh copy the stale persistent triangulations are dropped.
    assert!(!f.graph.mesh().persistent().faces().has(f.faces[0]));
}

#[test]
fn a_mesh_copy_transforms_one_deduplicated_copy_per_source_handle() {
    let mut f = fixture();
    transform_whole_graph(&mut f.graph, &translation(), true, true).unwrap();
    let copy_a = f
        .graph
        .mesh()
        .persistent()
        .faces()
        .triangulation(f.faces[0]);
    let copy_b = f
        .graph
        .mesh()
        .persistent()
        .faces()
        .triangulation(f.faces[1]);
    assert!(copy_a == copy_b);
    assert!(copy_a != f.triangulation);
    assert_eq!(copy_a.node(2).unwrap(), Point3::new(2.0, 2.0, 3.0));
    assert_eq!(f.triangulation.node(2).unwrap(), Point3::new(1.0, 0.0, 0.0));
    assert_eq!(copy_a.deflection(), 0.5);
}

#[test]
fn a_negative_transform_flips_the_winding_of_the_copied_mesh() {
    let mut f = fixture();
    assert!(requires_geometry_modification(&mirror(), false));
    transform_whole_graph(&mut f.graph, &mirror(), true, true).unwrap();
    let copy = f
        .graph
        .mesh()
        .persistent()
        .faces()
        .triangulation(f.faces[0]);
    assert_eq!(copy.triangle(1).unwrap().get(), (1, 3, 2));
    assert_eq!(copy.node(3).unwrap(), Point3::new(0.0, 1.0, 0.0));
}

#[test]
fn a_location_transform_composes_into_the_root_occurrence_references() {
    let mut f = fixture();
    f.graph
        .editor()
        .products()
        .append_document_root(f.product)
        .unwrap();
    assert!(!requires_geometry_modification(&translation(), false));
    transform_whole_graph(&mut f.graph, &translation(), false, true).unwrap();
    let reference = f
        .graph
        .topo()
        .products()
        .relations(f.product)
        .occurrence_ref_ids[0];
    let placement = f
        .graph
        .refs()
        .occurrences()
        .entry(reference)
        .local_location
        .clone();
    let moved = Point3::default().transformed(placement.transformation());
    assert_eq!(moved, Point3::new(1.0, 2.0, 3.0));
    // Location mode leaves the points and copies the mesh as-is.
    assert_eq!(point_of(&f.graph, 1), Point3::new(1.0, 0.0, 0.0));
    let copy = f
        .graph
        .mesh()
        .persistent()
        .faces()
        .triangulation(f.faces[0]);
    assert!(copy != f.triangulation);
    assert_eq!(copy.node(2).unwrap(), Point3::new(1.0, 0.0, 0.0));
}

#[test]
fn a_copied_range_transform_leaves_the_earlier_entities() {
    let mut f = fixture();
    let before = CopiedRange {
        nb_vertices: 1,
        nb_edges: 1,
        nb_co_edges: 0,
        nb_faces: 1,
        nb_products: 0,
    };
    let polygon = Handle::new(Polygon3D::new(2, false).unwrap());
    let mut edges = f.graph.editor().edges();
    edges.set_persistent_polygon3d(f.edge, &polygon).unwrap();
    transform_copied_range(&mut f.graph, &before, &translation(), true, true).unwrap();
    assert_eq!(point_of(&f.graph, 0), Point3::new(0.0, 0.0, 0.0));
    assert_eq!(point_of(&f.graph, 1), Point3::new(2.0, 2.0, 3.0));
    // The copied range owns fresh mesh copies, transformed in place: here the one
    // triangulation both faces share, reached through the second face.
    assert_eq!(f.triangulation.node(2).unwrap(), Point3::new(2.0, 2.0, 3.0));
    assert_eq!(polygon.deflection(), 0.0);
}

#[test]
fn the_range_of_a_graph_counts_its_definitions() {
    let f = fixture();
    let range = CopiedRange::of(&f.graph);
    assert!(!range.contains(FaceId::new(1)));
    assert!(range.contains(FaceId::new(2)));
    assert!(CopiedRange::default().contains(VertexId::new(0)));
    let product: NodeId = f.product.into();
    assert!(product.is_valid());
}

#[test]
fn a_scaling_transform_needs_geometry_modification() {
    let mut scale = Transform3::default();
    scale.set_scale(Point3::default(), 2.0).unwrap();
    assert!(is_scaled(&scale));
    assert!(requires_geometry_modification(&scale, false));
    assert!(requires_geometry_modification(&Transform3::default(), true));
    assert!(!requires_geometry_modification(
        &Transform3::default(),
        false
    ));
}
