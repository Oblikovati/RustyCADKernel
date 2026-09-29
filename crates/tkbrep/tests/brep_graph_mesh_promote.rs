//! Behaviour of `MeshView::EditorView::PromoteToPersistent` on a box graph: fresh cache
//! entries become persistent representations through the graph editor, stale ones stay
//! behind, and each write notifies as the editor setter does.
mod support;
use rustycad_tkbrep::brep_graph::editor::FaceOps;
use rustycad_tkbrep::brep_graph::mesh_view::editor_view::EditorView;
use rustycad_tkbrep::brep_graph::refs_iterator::CoEdgesOfWire;
use rustycad_tkbrep::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId, WireId};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};
use support::make_box;

fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(&make_box(10.0, 20.0, 30.0)).unwrap();
    graph
}

fn triangulation() -> Handle<Triangulation> {
    Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap())
}

/// The first coedge of the first wire, and the face it lies on.
fn first_co_edge(graph: &BRepGraph) -> (CoEdgeId, FaceId) {
    let co_edge = CoEdgesOfWire::new(graph, WireId::start()).current_id();
    (co_edge, graph.topo().co_edges().definition(co_edge).face_id)
}

#[test]
fn fresh_face_and_edge_entries_become_persistent() {
    let mut graph = box_graph();
    let (face, edge) = (FaceId::new(1), EdgeId::new(2));
    let tri = triangulation();
    let polygon = Handle::new(Polygon3D::new(2, false).unwrap());
    let cache = graph.mesh().editor();
    cache.faces().set_cached_triangulation(face, &tri).unwrap();
    cache.edges().set_cached_polygon3d(edge, &polygon).unwrap();
    assert!(!graph.mesh().persistent().faces().has(face));
    let own_gen = graph.topo().faces().definition(face).base.own_gen;

    EditorView::promote_to_persistent(&mut graph).unwrap();

    assert!(graph.mesh().persistent().faces().triangulation(face) == tri);
    assert!(graph.mesh().persistent().edges().polygon3d(edge) == polygon);
    assert!(!graph.mesh().persistent().faces().has(FaceId::new(0)));
    assert_eq!(
        graph.topo().faces().definition(face).base.own_gen,
        own_gen + 1
    );
}

#[test]
fn a_stale_face_entry_is_not_promoted() {
    let mut graph = box_graph();
    let face = FaceId::new(0);
    let cache = graph.mesh().editor();
    cache
        .faces()
        .set_cached_triangulation(face, &triangulation())
        .unwrap();
    let mut guard = graph.editor().faces().mutate(face).unwrap();
    FaceOps::set_tolerance_guarded(&mut guard, 1.0);
    drop(guard);

    EditorView::promote_to_persistent(&mut graph).unwrap();

    assert!(!graph.mesh().persistent().faces().has(face));
}

#[test]
fn a_co_edge_promotes_its_polygon2d_and_polygon_on_triangulation() {
    let mut graph = box_graph();
    let (co_edge, _face) = first_co_edge(&graph);
    let polygon2d = Handle::new(Polygon2D::new(2).unwrap());
    let on_tri = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    let cache = graph.mesh().editor();
    cache
        .co_edges()
        .set_cached_polygon2d(co_edge, &polygon2d)
        .unwrap();
    cache
        .co_edges()
        .append_cached_polygon_on_tri(co_edge, &on_tri)
        .unwrap();

    EditorView::promote_to_persistent(&mut graph).unwrap();

    let persistent = graph.mesh().persistent().co_edges();
    assert!(persistent.polygon_on_surface(co_edge) == polygon2d);
    // The face was never meshed in the cache, so the polygon on triangulation is stale
    // from the start (CacheMesh snapshot 0 against mesh generation 0) and stays behind.
    assert!(!persistent.has_polygon_on_triangulation(co_edge));
}

#[test]
fn a_promoted_face_leaves_its_coedge_polygons_on_triangulation_behind() {
    let mut graph = box_graph();
    let (co_edge, face) = first_co_edge(&graph);
    let tri = triangulation();
    let on_tri = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    let cache = graph.mesh().editor();
    cache.faces().set_cached_triangulation(face, &tri).unwrap();
    cache
        .co_edges()
        .append_cached_polygon_on_tri(co_edge, &on_tri)
        .unwrap();
    assert!(
        graph
            .mesh()
            .cache()
            .co_edges()
            .find_polygon_on_tri(co_edge)
            .is_some()
    );

    EditorView::promote_to_persistent(&mut graph).unwrap();

    // The face write came first and notified the face, which made the coedge entry
    // stale before the coedge loop read it (as natively).
    assert!(graph.mesh().persistent().faces().triangulation(face) == tri);
    assert!(
        !graph
            .mesh()
            .persistent()
            .co_edges()
            .has_polygon_on_triangulation(co_edge)
    );
}
