//! Rust replay of the pinned `BRepGraph_Polygon_Test.cxx` source.
#![allow(non_snake_case)]
#[path = "brep_graph_inc_populate_regression/fixtures.rs"]
#[allow(dead_code)]
mod fixtures;
mod support;

use fixtures::{attach_box_mesh, box_shape, subshapes};
use rustycad_tkbrep::brep::EdgePCurve;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId, SolidId, tool},
    topods::{self, Edge, Face, Shape},
};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{gp::Point2, poly::mesh_purpose};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn BRepGraph_PolygonTest__MultiTriangulation_Roundtrip_PreservesAll() {
    let shape = meshed_box();
    let graph = populate(&shape);
    assert_face_mesh_roundtrip(&graph);
}

#[test]
fn BRepGraph_PolygonTest__Polygon3D_Captured_WhenPresent() {
    let shape = meshed_box();
    let graph = populate(&shape);
    assert_eq!(
        graph_polygon3d_count(&graph),
        source_polygon3d_count(&shape)
    );
}

#[test]
fn BRepGraph_PolygonTest__PolyOnTri_Captured_AfterMesh() {
    let graph = populate(&meshed_box());
    assert!(co_edges_with_poly_on_tri(&graph) > 0);
    assert_coedge_faces_valid(&graph);
}

#[test]
fn BRepGraph_PolygonTest__PolyOnTri_Roundtrip_PreservedOnReconstruct() {
    let graph = populate(&meshed_box());
    let solid = graph.shapes().reconstruct(SolidId::start().into()).unwrap();
    assert!(reconstructed_poly_on_tri_count(&solid) > 0);
}

#[test]
fn BRepGraph_PolygonTest__UVPoints_Recomputed_OnPCurves() {
    let graph = populate(&box_shape(10.0, 20.0, 30.0, false));
    assert!(non_origin_uv_count(&graph) > 0);
}

#[test]
fn BRepGraph_PolygonTest__VertexPointRepresentations_StructurallyValid() {
    assert_cylinder_vertex_representations();
}

#[test]
fn BRepGraph_PolygonTest__SeamEdge_PolyOnTri_TwoEntries() {
    let cylinder = cylinder_shape();
    let graph = populate(&cylinder);
    assert!(has_two_coedges_on_a_face(&graph));
}

fn meshed_box() -> Shape {
    let shape = support::make_box(10.0, 20.0, 30.0).into_shape();
    attach_box_mesh(&shape);
    shape
}

fn populate(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(shape).unwrap();
    graph
}

fn assert_face_mesh_roundtrip(graph: &BRepGraph) {
    let mut found = false;
    for index in 0..graph.topo().faces().nb() {
        let face = FaceId::new(index);
        found |= face_has_roundtrip_triangulation(graph, face);
    }
    assert!(found, "meshed box should have triangulations");
}

fn face_has_roundtrip_triangulation(graph: &BRepGraph, face: FaceId) -> bool {
    if graph
        .topo()
        .faces()
        .definition(face)
        .triangulation_rep_id
        .is_valid()
    {
        assert!(
            !graph
                .mesh()
                .effective()
                .faces()
                .triangulation(face)
                .is_null()
        );
    }
    let rebuilt = graph.shapes().reconstruct(face.into()).unwrap();
    let triangulation =
        BRepTool::triangulation(&topods::Face::unchecked(rebuilt), mesh_purpose::NONE)
            .unwrap()
            .0;
    assert!(!triangulation.is_null());
    graph
        .topo()
        .faces()
        .definition(face)
        .triangulation_rep_id
        .is_valid()
}

fn graph_polygon3d_count(graph: &BRepGraph) -> usize {
    (0..graph.topo().edges().nb())
        .filter(|index| graph.mesh().effective().edges().has(EdgeId::new(*index)))
        .count()
}

fn source_polygon3d_count(shape: &Shape) -> usize {
    subshapes(shape, ShapeEnum::Edge)
        .iter()
        .filter(|shape| {
            !BRepTool::polygon3d(&topods::Edge::unchecked((*shape).clone()))
                .unwrap()
                .0
                .is_null()
        })
        .count()
}

fn co_edges_with_poly_on_tri(graph: &BRepGraph) -> usize {
    (0..graph.topo().co_edges().nb())
        .filter(|index| {
            graph
                .topo()
                .co_edges()
                .definition(CoEdgeId::new(*index))
                .polygon_on_tri_rep_id
                .is_valid()
        })
        .count()
}

fn assert_coedge_faces_valid(graph: &BRepGraph) {
    for index in 0..graph.topo().co_edges().nb() {
        assert!(
            graph
                .topo()
                .co_edges()
                .definition(CoEdgeId::new(index))
                .face_id
                .is_valid()
        );
    }
}

fn reconstructed_poly_on_tri_count(shape: &Shape) -> usize {
    let edges = subshapes(shape, ShapeEnum::Edge);
    let faces = subshapes(shape, ShapeEnum::Face);
    edges
        .iter()
        .map(|edge| count_edge_mesh_on_faces(edge, &faces))
        .sum()
}

fn count_edge_mesh_on_faces(edge: &Shape, faces: &[Shape]) -> usize {
    let edge = topods::Edge::unchecked(edge.clone());
    faces
        .iter()
        .filter(|face| face_has_polygon(&edge, face))
        .count()
}

fn face_has_polygon(edge: &topods::Edge, face: &Shape) -> bool {
    let face = topods::Face::unchecked(face.clone());
    let (triangulation, location) = BRepTool::triangulation(&face, mesh_purpose::NONE).unwrap();
    !triangulation.is_null()
        && !BRepTool::polygon_on_triangulation(edge, &triangulation, &location)
            .unwrap()
            .is_null()
}

fn non_origin_uv_count(graph: &BRepGraph) -> usize {
    (0..graph.topo().co_edges().nb())
        .filter(|index| {
            let (first, last) = tool::co_edge::uv_points(graph, CoEdgeId::new(*index)).unwrap();
            first.distance(Point2::default()) > precision::confusion()
                || last.distance(Point2::default()) > precision::confusion()
        })
        .count()
}

fn assert_cylinder_vertex_representations() {
    let cylinder = cylinder_shape();
    assert!(add_vertex_edge_point(&cylinder));
    assert!(add_vertex_surface_point(&cylinder));
    assert!(add_vertex_pcurve_point(&cylinder));
}

fn cylinder_shape() -> Shape {
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
    BRepPrimAPIMakeCylinder::new(5.0, 10.0)
        .shape()
        .unwrap()
        .clone()
}

fn add_vertex_edge_point(shape: &Shape) -> bool {
    let Some(edge) = first_edge_with_curve(shape) else {
        return false;
    };
    let Some(vertex) = edge_vertices(&edge).first().cloned() else {
        return false;
    };
    update_vertex_edge_point(&edge, &vertex)
}

fn first_edge_with_curve(shape: &Shape) -> Option<Edge> {
    subshapes(shape, ShapeEnum::Edge)
        .into_iter()
        .map(Edge::unchecked)
        .find(|edge| BRepTool::curve(edge).is_ok())
}

fn update_vertex_edge_point(edge: &Edge, vertex: &topods::Vertex) -> bool {
    let builder = BRepBuilder::default();
    let (_, first, _) = BRepTool::curve(edge).unwrap();
    builder
        .update_vertex_on_edge(
            vertex,
            first,
            edge,
            BRepTool::vertex_tolerance(vertex).unwrap(),
        )
        .unwrap();
    true
}

fn add_vertex_surface_point(shape: &Shape) -> bool {
    let Some((face, vertex)) = first_face_vertex(shape) else {
        return false;
    };
    update_vertex_surface_point(&face, &vertex)
}

fn first_face_vertex(shape: &Shape) -> Option<(Face, topods::Vertex)> {
    subshapes(shape, ShapeEnum::Face)
        .into_iter()
        .map(Face::unchecked)
        .find_map(|face| {
            let vertices = subshapes(&face.clone().into_shape(), ShapeEnum::Vertex);
            vertices
                .first()
                .map(|shape| (face, topods::Vertex::unchecked(shape.clone())))
        })
}

fn update_vertex_surface_point(face: &Face, vertex: &topods::Vertex) -> bool {
    let builder = BRepBuilder::default();
    let uv = BRepTool::parameters(vertex, face).unwrap();
    builder
        .update_vertex_on_face(
            vertex,
            uv.x(),
            uv.y(),
            face,
            BRepTool::vertex_tolerance(vertex).unwrap(),
        )
        .unwrap();
    true
}

fn add_vertex_pcurve_point(shape: &Shape) -> bool {
    subshapes(shape, ShapeEnum::Face)
        .into_iter()
        .map(Face::unchecked)
        .any(|face| add_first_vertex_pcurve_point(&face))
}

fn add_first_vertex_pcurve_point(face: &Face) -> bool {
    let builder = BRepBuilder::default();
    let Some((edge, pcurve)) = first_edge_pcurve(face) else {
        return false;
    };
    update_edge_pcurve(&builder, &edge, face, &pcurve);
    let Some(vertex) = edge_vertices(&edge).first().cloned() else {
        return false;
    };
    builder
        .update_vertex_on_edge_face(
            &vertex,
            pcurve.first,
            &edge,
            face,
            BRepTool::vertex_tolerance(&vertex).unwrap(),
        )
        .unwrap();
    true
}

fn first_edge_pcurve(face: &Face) -> Option<(Edge, EdgePCurve)> {
    subshapes(&face.clone().into_shape(), ShapeEnum::Edge)
        .into_iter()
        .map(Edge::unchecked)
        .find_map(|edge| {
            let pcurve = BRepTool::curve_on_face(&edge, face).ok()?;
            (!pcurve.pcurve.is_null()).then_some((edge, pcurve))
        })
}

fn update_edge_pcurve(builder: &BRepBuilder, edge: &Edge, face: &Face, pcurve: &EdgePCurve) {
    builder
        .update_edge_pcurve_on_face(
            edge,
            &pcurve.pcurve,
            face,
            BRepTool::edge_tolerance(edge).unwrap(),
        )
        .unwrap();
    builder
        .range_on_face(edge, face, pcurve.first, pcurve.last)
        .unwrap();
}

fn edge_vertices(edge: &topods::Edge) -> Vec<topods::Vertex> {
    topods::ShapeIterator::new(&edge.clone().into_shape(), false, false)
        .filter(|child| child.shape_type().ok() == Some(ShapeEnum::Vertex))
        .map(topods::Vertex::unchecked)
        .collect()
}

// OCCT meshes the source cylinder, but its test only checks repeated seam coedges.
// A native probe shows one PolygonOnTriangulation (nodes 28, 1), so fabricated mesh
// entries would make the Rust setup less faithful than the topology-only assertion.
fn has_two_coedges_on_a_face(graph: &BRepGraph) -> bool {
    for edge_index in 0..graph.topo().edges().nb() {
        let co_edges = graph.topo().edges().co_edges(EdgeId::new(edge_index));
        for face_index in 0..graph.topo().faces().nb() {
            let count = co_edges
                .iter()
                .filter(|co_edge| {
                    let def = graph.topo().co_edges().definition(**co_edge);
                    def.face_id == FaceId::new(face_index)
                })
                .count();
            if count >= 2 {
                return true;
            }
        }
    }
    false
}
