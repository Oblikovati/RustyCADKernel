#[test]
fn BRepGraph_TransformTest__CopyMesh_TriangulationNodesTransformed() {
    let mut graph = transform_box(true);
    assert!(graph.topo().faces().nb() >= 1);
    let tri = transform_triangle(
        [
            Point3::new(1.0, 2.0, 3.0),
            Point3::new(4.0, 5.0, 6.0),
            Point3::new(7.0, 8.0, 9.0),
        ],
        Some(0.1),
    );
    graph
        .editor()
        .faces()
        .set_persistent_triangulation(FaceId::start(), &tri)
        .unwrap();
    graph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(FaceId::start(), &tri)
        .unwrap();
    let result = transformed_graph(
        &graph,
        &graph_translation(5.0, 10.0, 15.0),
        GeomPolicy::Copy,
        MeshPolicy::Copy,
    );
    assert_translated_triangulation(&result, &tri);
}
fn assert_translated_triangulation(result: &BRepGraph, source: &Handle<Triangulation>) {
    let tri = result
        .mesh()
        .persistent()
        .faces()
        .triangulation(FaceId::start());
    assert!(!tri.is_null());
    let mesh = tri.get().unwrap();
    assert_eq!(mesh.nb_nodes(), 3);
    transform_point_near(mesh.node(1).unwrap(), Point3::new(6.0, 12.0, 18.0));
    transform_near(mesh.node(3).unwrap().x(), 12.0);
    transform_near(source.get().unwrap().node(1).unwrap().x(), 1.0);
    transform_near(mesh.deflection(), 0.1);
}
#[test]
fn BRepGraph_TransformTest__CopyMesh_False_TriangulationInvalidated() {
    let mut graph = transform_box(true);
    let tri = transform_planar_triangle(0.0, 1.0, None);
    graph
        .editor()
        .faces()
        .set_persistent_triangulation(FaceId::start(), &tri)
        .unwrap();
    let result = transformed_graph(
        &graph,
        &graph_translation(1.0, 0.0, 0.0),
        GeomPolicy::Copy,
        MeshPolicy::Drop,
    );
    assert!(
        result
            .mesh()
            .persistent()
            .faces()
            .triangulation(FaceId::start())
            .is_null()
    );
    assert!(!result.mesh().effective().faces().has(FaceId::start()));
}
#[test]
fn BRepGraph_TransformTest__TransformNode_CopyGeomAndMesh_DropsRuntimeCache() {
    let graph = transform_source(&support::make_box(10.0, 10.0, 10.0), false);
    assert!(graph.topo().faces().nb() >= 1);
    let tri = transform_planar_triangle(0.0, 2.0, Some(1.0));
    graph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(FaceId::start(), &tri)
        .unwrap();
    let result = transformed_node(
        &graph,
        SolidId::start().into(),
        &graph_translation(5.0, 0.0, 0.0),
        MeshPolicy::Copy,
    );
    assert!(
        result
            .mesh()
            .cache()
            .faces()
            .entry(FaceId::start())
            .is_none()
    );
}
#[test]
fn BRepGraph_TransformTest__TransformNode_CopyGeomAndMesh_CopiesPersistentMeshAndDropsCache() {
    let mut graph = transform_source(&support::make_box(10.0, 10.0, 10.0), false);
    let (coedge, edge) = checked_transform_mesh_ids(&graph);
    // Superseded representations must not be revived while copying the live mesh.
    write_transform_mesh(&mut graph, coedge, edge, &obsolete_transform_mesh());
    let mesh = active_transform_mesh();
    write_transform_mesh(&mut graph, coedge, edge, &mesh);
    cache_transform_mesh(&graph, coedge, edge, &mesh);
    let result = transformed_node(
        &graph,
        SolidId::start().into(),
        &graph_translation(5.0, 0.0, 0.0),
        MeshPolicy::Copy,
    );
    assert_transformed_persistent_mesh(&result);
}
fn assert_transformed_persistent_mesh(result: &BRepGraph) {
    let (coedge, edge) = checked_transform_mesh_ids(result);
    assert!(result.mesh().cache().edges().entry(edge).is_none());
    let polygon = result.mesh().persistent().edges().polygon3d(edge);
    assert!(!polygon.is_null());
    transform_near(polygon.get().unwrap().nodes().value(1).unwrap().x(), 6.0);
    assert!(!result.mesh().cache().co_edges().has(coedge));
    let pcurve = result
        .mesh()
        .persistent()
        .co_edges()
        .polygon_on_surface(coedge);
    assert!(!pcurve.is_null());
    transform_near(pcurve.get().unwrap().nodes().value(1).unwrap().x(), 7.0);
    assert_transformed_face_mesh(result, coedge, edge);
}
fn assert_transformed_face_mesh(result: &BRepGraph, coedge: CoEdgeId, edge: EdgeId) {
    let face = result.topo().co_edges().definition(coedge).face_id;
    assert!(face.is_valid_for(result.topo().faces().nb()));
    let tri = result.mesh().persistent().faces().triangulation(face);
    assert!(!tri.is_null());
    transform_near(tri.get().unwrap().node(2).unwrap().x(), 6.0);
    let polygon = result
        .mesh()
        .persistent()
        .edges()
        .polygon_on_triangulation(edge, face);
    assert!(!polygon.is_null());
    let persistent = result.mesh().persistent().faces().triangulation(face);
    assert!(!persistent.is_null());
    transform_near(persistent.get().unwrap().node(2).unwrap().x(), 6.0);
}
