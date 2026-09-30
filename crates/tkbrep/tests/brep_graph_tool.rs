//! Original BRepGraph_Tool_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_graph::{
    BRepGraph, CoEdgeId, EdgeId, FaceId, ShellId, VertexId, WireId,
    tool::{co_edge, edge, face, shell, vertex, wire},
};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkmath::poly::{PolygonOnTriangulation, Triangulation};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn query_surface_graph(maker: &mut impl BRepBuilderAPIMakeShape) -> BRepGraph {
    let shape = maker.shape().unwrap().clone();
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(&shape).unwrap();
    graph
}
fn query_surface_fixture() -> (BRepGraph, BRepGraph) {
    let box_graph =
        query_surface_graph(&mut BRepPrimAPIMakeBox::from_sizes(10.0, 20.0, 30.0).unwrap());
    let cylinder = query_surface_graph(&mut BRepPrimAPIMakeCylinder::new(5.0, 15.0));
    (box_graph, cylinder)
}
fn query_triangulation() -> Handle<Triangulation> {
    Handle::new(Triangulation::with_sizes(1, 1, false, false).unwrap())
}
fn set_query_polygon(
    graph: &mut BRepGraph,
    coedge: CoEdgeId,
    face: FaceId,
) -> Handle<PolygonOnTriangulation> {
    graph
        .editor()
        .faces()
        .set_persistent_triangulation(face, &query_triangulation())
        .unwrap();
    let polygon = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    graph
        .editor()
        .co_edges()
        .set_persistent_polygon_on_tri(coedge, &polygon)
        .unwrap();
    polygon
}
#[test]
fn BRepGraph_QuerySurfaceTest__Face_NbWires_BoxFaceHasOneWire() {
    let (graph, _cylinder) = query_surface_fixture();
    for i in 0..graph.topo().faces().nb() {
        assert_eq!(face::nb_wires(&graph, FaceId::new(i)), 1);
    }
}
#[test]
fn BRepGraph_QuerySurfaceTest__Face_Bounds_BoxFaceHasFiniteBounds() {
    let (graph, _cylinder) = query_surface_fixture();
    let (umin, umax, vmin, vmax) = face::bounds(&graph, FaceId::new(0));
    assert!(umin < umax);
    assert!(vmin < vmax);
    assert!(umin < precision::infinite());
    assert!(vmin < precision::infinite());
}
#[test]
fn BRepGraph_QuerySurfaceTest__Face_Bounds_CylinderFaceReturnsSurfaceBounds() {
    let (_box_graph, graph) = query_surface_fixture();
    for i in 0..graph.topo().faces().nb() {
        let id = FaceId::new(i);
        if !face::has_surface(&graph, id) {
            continue;
        }
        assert_eq!(
            face::bounds(&graph, id),
            face::surface(&graph, id).get().unwrap().bounds()
        );
    }
}
#[test]
fn BRepGraph_QuerySurfaceTest__Wire_FaceOf_ReturnsValidFace() {
    let (graph, _cylinder) = query_surface_fixture();
    assert!(wire::face_of(&graph, WireId::new(0)).is_valid());
}
#[test]
fn BRepGraph_QuerySurfaceTest__Wire_IsOuter_FirstWireOfBoxFaceIsOuter() {
    let (graph, _cylinder) = query_surface_fixture();
    let outer = face::outer_wire(&graph, FaceId::new(0)).unwrap();
    assert!(outer.is_valid());
    assert!(wire::is_outer(&graph, outer).unwrap());
}
#[test]
fn BRepGraph_QuerySurfaceTest__Edge_NbFaces_BoxEdgeHasExactlyTwoFaces() {
    let (graph, _cylinder) = query_surface_fixture();
    for i in 0..graph.topo().edges().nb() {
        assert_eq!(edge::nb_faces(&graph, EdgeId::new(i)), 2);
    }
}
#[test]
fn BRepGraph_QuerySurfaceTest__Edge_IsManifold_BoxEdgesAreManifold() {
    let (graph, _cylinder) = query_surface_fixture();
    for i in 0..graph.topo().edges().nb() {
        assert!(edge::is_manifold(&graph, EdgeId::new(i)));
        assert!(!edge::is_boundary(&graph, EdgeId::new(i)));
    }
}
#[test]
fn BRepGraph_QuerySurfaceTest__Edge_IsClosed_BoxEdgesAreOpen() {
    let (graph, _cylinder) = query_surface_fixture();
    let count = graph.topo().edges().nb();
    assert!(count > 0);
    for i in 0..count {
        assert!(!edge::is_closed(&graph, EdgeId::new(i)));
    }
}
#[test]
fn BRepGraph_QuerySurfaceTest__Edge_IsClosed_DerivedFromVertexTopology() {
    let (graph, _cylinder) = query_surface_fixture();
    let id = EdgeId::new(0);
    assert!(id.is_valid_for(graph.topo().edges().nb()));
    assert!(!edge::is_closed(&graph, id));
}
#[test]
fn BRepGraph_QuerySurfaceTest__CoEdge_PolygonOnTriangulation_RoundTrip() {
    let (mut graph, _cylinder) = query_surface_fixture();
    assert!(graph.topo().co_edges().nb() > 0);
    let id = CoEdgeId::new(0);
    let face = co_edge::face_of(&graph, id);
    assert!(
        !graph
            .mesh()
            .persistent()
            .co_edges()
            .has_polygon_on_triangulation(id)
    );
    assert!(
        graph
            .mesh()
            .persistent()
            .co_edges()
            .polygon_on_triangulation(id)
            .is_null()
    );
    let polygon = set_query_polygon(&mut graph, id, face);
    assert!(
        graph
            .mesh()
            .persistent()
            .co_edges()
            .has_polygon_on_triangulation(id)
    );
    assert!(
        graph
            .mesh()
            .persistent()
            .co_edges()
            .polygon_on_triangulation(id)
            == polygon
    );
}
#[test]
fn BRepGraph_QuerySurfaceTest__Edge_PolygonOnTriangulation_ResolvesViaFace() {
    let (mut graph, _cylinder) = query_surface_fixture();
    assert!(graph.topo().co_edges().nb() > 0);
    let id = CoEdgeId::new(0);
    let edge = co_edge::edge_of(&graph, id);
    let face = co_edge::face_of(&graph, id);
    assert!(edge.is_valid());
    assert!(face.is_valid());
    assert!(
        !graph
            .mesh()
            .persistent()
            .edges()
            .has_polygon_on_triangulation(edge, face)
    );
    let polygon = set_query_polygon(&mut graph, id, face);
    assert!(
        graph
            .mesh()
            .persistent()
            .edges()
            .has_polygon_on_triangulation(edge, face)
    );
    assert!(
        graph
            .mesh()
            .persistent()
            .edges()
            .polygon_on_triangulation(edge, face)
            == polygon
    );
}
#[test]
fn BRepGraph_QuerySurfaceTest__Face_CachedTriangulation_DefaultEmpty() {
    let (graph, _cylinder) = query_surface_fixture();
    for i in 0..graph.topo().faces().nb() {
        let id = FaceId::new(i);
        assert!(!graph.mesh().cache().faces().has(id));
        assert!(graph.mesh().cache().faces().triangulation(id).is_null());
    }
}
#[test]
fn BRepGraph_QuerySurfaceTest__Face_CachedTriangulation_SetAndRead() {
    let (graph, _cylinder) = query_surface_fixture();
    assert!(graph.topo().faces().nb() > 0);
    let id = FaceId::new(0);
    let tri = query_triangulation();
    graph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(id, &tri)
        .unwrap();
    assert!(graph.mesh().cache().faces().has(id));
    assert!(graph.mesh().cache().faces().triangulation(id) == tri);
}
#[test]
fn BRepGraph_QuerySurfaceTest__Vertex_NbEdges_BoxVertexHasThreeEdges() {
    let (graph, _cylinder) = query_surface_fixture();
    for i in 0..graph.topo().vertices().nb() {
        assert_eq!(vertex::nb_edges(&graph, VertexId::new(i)), 3);
    }
}
#[test]
fn BRepGraph_QuerySurfaceTest__Shell_IsClosed_BoxShellIsClosed() {
    let (graph, _cylinder) = query_surface_fixture();
    assert!(graph.topo().shells().nb() >= 1);
    assert!(shell::is_closed(&graph, ShellId::start()));
}
#[test]
fn BRepGraph_QuerySurfaceTest__Shell_NbFaces_BoxShellHasSixFaces() {
    let (graph, _cylinder) = query_surface_fixture();
    assert!(graph.topo().shells().nb() >= 1);
    assert_eq!(shell::nb_faces(&graph, ShellId::start()), 6);
}
