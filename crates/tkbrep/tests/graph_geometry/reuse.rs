fn geometry_owned_edge(graph: &BRepGraph) -> EdgeId {
    let edges = graph.topo().edges();
    let id = geometry_ids::<EdgeTag>(graph)
        .into_iter()
        .find(|id| edges.definition(*id).curve3d_rep_id.is_valid())
        .unwrap();
    assert!(id.is_valid_for(edges.nb()));
    assert!(!edge::curve(graph, id).is_null());
    id
}
fn geometry_owned_coedge(graph: &BRepGraph) -> CoEdgeId {
    let coedges = graph.topo().co_edges();
    let id = geometry_ids::<CoEdgeTag>(graph)
        .into_iter()
        .find(|id| coedges.definition(*id).curve2d_rep_id.is_valid())
        .unwrap();
    assert!(id.is_valid_for(coedges.nb()));
    assert!(!co_edge::pcurve(graph, id).is_null());
    id
}
fn geometry_owned_ids(graph: &BRepGraph) -> (FaceId, EdgeId, CoEdgeId) {
    let id = FaceId::start();
    assert!(id.is_valid_for(graph.topo().faces().nb()));
    let rep = geometry_surface_state(graph, id).0;
    assert!(rep.is_valid());
    assert!(!face::surface(graph, id).is_null());
    (id, geometry_owned_edge(graph), geometry_owned_coedge(graph))
}
fn geometry_reuse_surface(graph: &mut BRepGraph, id: FaceId) {
    let rep = geometry_surface_state(graph, id).0;
    let handle = face::surface(graph, id);
    let active = geometry_surface_state(graph, id).1;
    assert!(active > 0);
    graph.editor().faces().clear_surface(id).unwrap();
    assert_eq!(geometry_surface_state(graph, id).0, rep);
    assert!(!face::has_surface(graph, id));
    assert_eq!(geometry_surface_state(graph, id).1, active - 1);
    graph.editor().faces().set_surface(id, &handle).unwrap();
    assert_eq!(geometry_surface_state(graph, id).0, rep);
    assert_eq!(geometry_surface_state(graph, id).1, active);
    graph.editor().faces().clear_surface(id).unwrap();
    assert_eq!(geometry_surface_state(graph, id).1, active - 1);
}
fn geometry_reuse_curve3d(graph: &mut BRepGraph, id: EdgeId) {
    let rep = geometry_curve3d_state(graph, id).0;
    let handle = edge::curve(graph, id);
    let active = geometry_curve3d_state(graph, id).1;
    assert!(active > 0);
    graph.editor().edges().clear_curve(id).unwrap();
    assert_eq!(geometry_curve3d_state(graph, id).0, rep);
    assert!(!edge::has_curve(graph, id));
    assert_eq!(geometry_curve3d_state(graph, id).1, active - 1);
    graph
        .editor()
        .edges()
        .set_curve(id, &handle, 0.0, 1.0)
        .unwrap();
    assert_eq!(geometry_curve3d_state(graph, id).0, rep);
    assert_eq!(geometry_curve3d_state(graph, id).1, active);
    graph.editor().edges().clear_curve(id).unwrap();
    assert_eq!(geometry_curve3d_state(graph, id).1, active - 1);
}
fn geometry_reuse_pcurve(graph: &mut BRepGraph, id: CoEdgeId) {
    let rep = geometry_pcurve_state(graph, id).0;
    let handle = co_edge::pcurve(graph, id);
    let active = geometry_pcurve_state(graph, id).1;
    assert!(active > 0);
    graph.editor().co_edges().clear_pcurve(id).unwrap();
    assert_eq!(geometry_pcurve_state(graph, id).0, rep);
    assert!(!co_edge::has_pcurve(graph, id));
    assert_eq!(geometry_pcurve_state(graph, id).1, active - 1);
    graph
        .editor()
        .co_edges()
        .set_pcurve(id, &handle, 0.0, 1.0)
        .unwrap();
    assert_eq!(geometry_pcurve_state(graph, id).0, rep);
    assert_eq!(geometry_pcurve_state(graph, id).1, active);
    graph.editor().co_edges().clear_pcurve(id).unwrap();
    assert_eq!(geometry_pcurve_state(graph, id).1, active - 1);
}
struct GeometryMeshOwners {
    triangulation: Handle<Triangulation>,
    polygon3d: Handle<Polygon3D>,
    polygon2d: Handle<Polygon2D>,
    polygon_on_tri: Handle<PolygonOnTriangulation>,
}
fn geometry_mesh_owners(
    graph: &mut BRepGraph,
    ids: (FaceId, EdgeId, CoEdgeId),
) -> GeometryMeshOwners {
    let owners = GeometryMeshOwners {
        triangulation: Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap()),
        polygon3d: Handle::new(Polygon3D::new(2, false).unwrap()),
        polygon2d: Handle::new(Polygon2D::new(2).unwrap()),
        polygon_on_tri: Handle::new(PolygonOnTriangulation::new(2, false).unwrap()),
    };
    geometry_set_triangulation(graph, ids.0, &owners.triangulation);
    geometry_set_polygon3d(graph, ids.1, &owners.polygon3d);
    geometry_set_polygon2d(graph, ids.2, &owners.polygon2d);
    geometry_set_polygon_on_tri(graph, ids.2, &owners.polygon_on_tri);
    owners
}
fn geometry_reuse_triangulation(graph: &mut BRepGraph, id: FaceId, owners: &GeometryMeshOwners) {
    let rep = geometry_triangulation_state(graph, id).0;
    let active = geometry_triangulation_state(graph, id).1;
    assert!(active > 0);
    geometry_clear_triangulation(graph, id);
    assert_eq!(geometry_triangulation_state(graph, id).0, rep);
    assert!(!graph.mesh().persistent().faces().has(id));
    assert_eq!(geometry_triangulation_state(graph, id).1, active - 1);
    geometry_set_triangulation(graph, id, &owners.triangulation);
    assert_eq!(geometry_triangulation_state(graph, id).0, rep);
    assert_eq!(geometry_triangulation_state(graph, id).1, active);
    geometry_clear_triangulation(graph, id);
}
fn geometry_reuse_polygon3d(graph: &mut BRepGraph, id: EdgeId, owners: &GeometryMeshOwners) {
    let rep = geometry_polygon3d_state(graph, id).0;
    let active = geometry_polygon3d_state(graph, id).1;
    assert!(active > 0);
    geometry_clear_polygon3d(graph, id);
    assert_eq!(geometry_polygon3d_state(graph, id).0, rep);
    assert!(!graph.mesh().persistent().edges().has(id));
    assert_eq!(geometry_polygon3d_state(graph, id).1, active - 1);
    geometry_set_polygon3d(graph, id, &owners.polygon3d);
    assert_eq!(geometry_polygon3d_state(graph, id).0, rep);
    assert_eq!(geometry_polygon3d_state(graph, id).1, active);
    geometry_clear_polygon3d(graph, id);
}
fn geometry_reuse_polygon2d(graph: &mut BRepGraph, id: CoEdgeId, owners: &GeometryMeshOwners) {
    let rep = geometry_polygon2d_state(graph, id).0;
    let active = geometry_polygon2d_state(graph, id).1;
    assert!(active > 0);
    geometry_set_polygon2d(graph, id, &Handle::default());
    assert_eq!(geometry_polygon2d_state(graph, id).0, rep);
    assert!(!graph.mesh().persistent().co_edges().has(id));
    assert_eq!(geometry_polygon2d_state(graph, id).1, active - 1);
    geometry_set_polygon2d(graph, id, &owners.polygon2d);
    assert_eq!(geometry_polygon2d_state(graph, id).0, rep);
    assert_eq!(geometry_polygon2d_state(graph, id).1, active);
    geometry_set_polygon2d(graph, id, &Handle::default());
}
fn geometry_reuse_polygon_on_tri(graph: &mut BRepGraph, id: CoEdgeId, owners: &GeometryMeshOwners) {
    let rep = geometry_polygon_on_tri_state(graph, id).0;
    let active = geometry_polygon_on_tri_state(graph, id).1;
    assert!(active > 0);
    geometry_set_polygon_on_tri(graph, id, &Handle::default());
    assert_eq!(geometry_polygon_on_tri_state(graph, id).0, rep);
    assert!(
        !graph
            .mesh()
            .persistent()
            .co_edges()
            .has_polygon_on_triangulation(id)
    );
    assert_eq!(geometry_polygon_on_tri_state(graph, id).1, active - 1);
    geometry_set_polygon_on_tri(graph, id, &owners.polygon_on_tri);
    assert_eq!(geometry_polygon_on_tri_state(graph, id).0, rep);
    assert_eq!(geometry_polygon_on_tri_state(graph, id).1, active);
}
#[test]
fn BRepGraph_GeometryTest__ClearUseOwnersMarksRecordsRemovedAndSetReusesSlots() {
    let mut graph = geometry_box(false);
    let ids = geometry_owned_ids(&graph);
    geometry_reuse_surface(&mut graph, ids.0);
    geometry_reuse_curve3d(&mut graph, ids.1);
    geometry_reuse_pcurve(&mut graph, ids.2);
    let owners = geometry_mesh_owners(&mut graph, ids);
    geometry_reuse_triangulation(&mut graph, ids.0, &owners);
    geometry_reuse_polygon3d(&mut graph, ids.1, &owners);
    geometry_reuse_polygon2d(&mut graph, ids.2, &owners);
    geometry_reuse_polygon_on_tri(&mut graph, ids.2, &owners);
}
