fn first_transform_coedge(graph: &BRepGraph, face: FaceId) -> CoEdgeId {
    for reference in RefsWireOfFace::new(graph, face) {
        let wire = graph.refs().wires().entry(reference).child_wire_id;
        for coedge in CoEdgesOfWire::new(graph, wire) {
            if graph.topo().co_edges().definition(coedge).face_id == face {
                return coedge;
            }
        }
    }
    CoEdgeId::default()
}
fn transform_triangle(points: [Point3; 3], deflection: Option<f64>) -> Handle<Triangulation> {
    let tri = Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap());
    for (index, point) in points.into_iter().enumerate() {
        tri.get()
            .unwrap()
            .set_node(index as i32 + 1, point)
            .unwrap();
    }
    tri.get()
        .unwrap()
        .set_triangle(1, Triangle::new(1, 2, 3))
        .unwrap();
    if let Some(value) = deflection {
        tri.get().unwrap().set_deflection(value);
    }
    tri
}
fn transform_planar_triangle(
    origin: f64,
    side: f64,
    deflection: Option<f64>,
) -> Handle<Triangulation> {
    transform_triangle(
        [
            Point3::new(origin, 0.0, 0.0),
            Point3::new(origin + side, 0.0, 0.0),
            Point3::new(origin, side, 0.0),
        ],
        deflection,
    )
}
fn transform_polygon3d(first: Point3, last: Point3) -> Handle<Polygon3D> {
    let polygon = Handle::new(Polygon3D::new(2, false).unwrap());
    polygon.get().unwrap().change_nodes(|nodes| {
        nodes.set_value(1, first).unwrap();
        nodes.set_value(2, last).unwrap();
    });
    polygon
}
fn transform_polygon2d(first: Point2, last: Point2) -> Handle<Polygon2D> {
    let polygon = Handle::new(Polygon2D::new(2).unwrap());
    polygon.get().unwrap().change_nodes(|nodes| {
        nodes.set_value(1, first).unwrap();
        nodes.set_value(2, last).unwrap();
    });
    polygon
}
fn transform_polygon_on_tri() -> Handle<PolygonOnTriangulation> {
    let polygon = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    polygon.get().unwrap().set_node(1, 1).unwrap();
    polygon.get().unwrap().set_node(2, 2).unwrap();
    polygon
}
fn checked_transform_mesh_ids(graph: &BRepGraph) -> (CoEdgeId, EdgeId) {
    let coedge = first_transform_coedge(graph, FaceId::start());
    assert!(coedge.is_valid_for(graph.topo().co_edges().nb()));
    let edge = graph.topo().co_edges().definition(coedge).child_edge_id;
    assert!(edge.is_valid_for(graph.topo().edges().nb()));
    (coedge, edge)
}
struct TransformMeshHandles {
    triangulation: Handle<Triangulation>,
    polygon3d: Handle<Polygon3D>,
    polygon2d: Handle<Polygon2D>,
    polygon_on_tri: Handle<PolygonOnTriangulation>,
}
fn obsolete_transform_mesh() -> TransformMeshHandles {
    TransformMeshHandles {
        triangulation: transform_planar_triangle(100.0, 1.0, None),
        polygon3d: transform_polygon3d(Point3::new(100.0, 0.0, 0.0), Point3::new(101.0, 0.0, 0.0)),
        polygon2d: transform_polygon2d(Point2::new(100.0, 0.0), Point2::new(101.0, 0.0)),
        polygon_on_tri: transform_polygon_on_tri(),
    }
}
fn active_transform_mesh() -> TransformMeshHandles {
    TransformMeshHandles {
        triangulation: transform_planar_triangle(0.0, 1.0, None),
        polygon3d: transform_polygon3d(Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 5.0, 6.0)),
        polygon2d: transform_polygon2d(Point2::new(7.0, 8.0), Point2::new(9.0, 10.0)),
        polygon_on_tri: transform_polygon_on_tri(),
    }
}
fn write_transform_mesh(
    graph: &mut BRepGraph,
    coedge: CoEdgeId,
    edge: EdgeId,
    mesh: &TransformMeshHandles,
) {
    graph
        .editor()
        .faces()
        .set_persistent_triangulation(FaceId::start(), &mesh.triangulation)
        .unwrap();
    graph
        .editor()
        .edges()
        .set_persistent_polygon3d(edge, &mesh.polygon3d)
        .unwrap();
    graph
        .editor()
        .co_edges()
        .set_persistent_polygon2d(coedge, &mesh.polygon2d)
        .unwrap();
    graph
        .editor()
        .co_edges()
        .set_persistent_polygon_on_tri(coedge, &mesh.polygon_on_tri)
        .unwrap();
}
fn cache_transform_mesh(
    graph: &BRepGraph,
    coedge: CoEdgeId,
    edge: EdgeId,
    mesh: &TransformMeshHandles,
) {
    let editor = graph.mesh().editor();
    editor
        .faces()
        .set_cached_triangulation(FaceId::start(), &mesh.triangulation)
        .unwrap();
    editor
        .edges()
        .set_cached_polygon3d(edge, &mesh.polygon3d)
        .unwrap();
    editor
        .co_edges()
        .set_cached_polygon2d(coedge, &mesh.polygon2d)
        .unwrap();
    editor
        .co_edges()
        .append_cached_polygon_on_tri(coedge, &mesh.polygon_on_tri)
        .unwrap();
}
