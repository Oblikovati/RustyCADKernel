#[test]
fn BRepGraph_TransformTest__TranslateBox_FaceCount() {
    let graph = transform_box(true);
    let transform = graph_translation(100.0, 200.0, 300.0);
    let result = transformed_graph(&graph, &transform, GeomPolicy::Copy, MeshPolicy::Drop);
    assert_eq!(result.topo().faces().nb(), 6);
    assert_eq!(result.topo().faces().nb(), graph.topo().faces().nb());
}
#[test]
fn BRepGraph_TransformTest__TranslateBox_AreaPreserved() {
    let shape = support::make_box(10.0, 20.0, 30.0);
    let original = transform_surface_props(&shape).mass();
    let graph = transform_source(&shape, true);
    let transform = graph_translation(50.0, 0.0, 0.0);
    let result = transformed_graph(&graph, &transform, GeomPolicy::Copy, MeshPolicy::Drop);
    let area = transform_face_area_sum(&result);
    assert!((area - original).abs() <= original * 0.01);
}
#[test]
fn BRepGraph_TransformTest__TranslateBox_VertexPointsShifted() {
    let graph = transform_box(true);
    assert!(graph.topo().vertices().nb() > 0);
    let transform = graph_translation(100.0, 200.0, 300.0);
    let result = transformed_graph(&graph, &transform, GeomPolicy::Copy, MeshPolicy::Drop);
    assert_eq!(result.topo().vertices().nb(), graph.topo().vertices().nb());
    for i in 0..graph.topo().vertices().nb() {
        let id = VertexId::new(i);
        let original = vertex::pnt(&graph, id);
        transform_point_near(
            vertex::pnt(&result, id),
            Point3::new(
                original.x() + 100.0,
                original.y() + 200.0,
                original.z() + 300.0,
            ),
        );
    }
}
#[test]
fn BRepGraph_TransformTest__PerformIntoNonEmptyTargetDoesNotMoveExistingVertices() {
    let graph = transform_box(true);
    let mut target = BRepGraph::new();
    let id = target
        .editor()
        .vertices()
        .add(Point3::new(1.0, 2.0, 3.0), 1e-7);
    let original = vertex::pnt(&target, id);
    let transform = graph_translation(100.0, 200.0, 300.0);
    assert!(
        transform::perform(
            &graph,
            &mut target,
            &transform,
            GeomPolicy::Copy,
            MeshPolicy::Drop
        )
        .unwrap()
    );
    transform_point_near(vertex::pnt(&target, id), original);
}
#[test]
fn BRepGraph_TransformTest__LocationOnly_NoCopyGeom() {
    let shape = support::make_box(10.0, 20.0, 30.0);
    let graph = transform_source(&shape, true);
    assert!(graph.topo().vertices().nb() > 0);
    let result = transformed_graph(
        &graph,
        &graph_translation(50.0, 0.0, 0.0),
        GeomPolicy::Share,
        MeshPolicy::Drop,
    );
    assert_eq!(result.topo().faces().nb(), 6);
    assert_eq!(result.topo().vertices().nb(), graph.topo().vertices().nb());
    for i in 0..graph.topo().vertices().nb() {
        transform_near(
            vertex::pnt(&result, VertexId::new(i)).x(),
            vertex::pnt(&graph, VertexId::new(i)).x(),
        );
    }
    assert_location_only_geometry(&result, &shape);
}
fn assert_location_only_geometry(graph: &BRepGraph, shape: &Shape) {
    assert!(graph.topo().products().nb() > 0);
    let reference = first_transform_occurrence(graph);
    let location = &graph.refs().occurrences().entry(reference).local_location;
    assert!(!location.is_identity());
    let transform = location.transformation();
    transform_near(transform.value(1, 4).unwrap(), 50.0);
    transform_near(transform.value(2, 4).unwrap(), 0.0);
    transform_near(transform.value(3, 4).unwrap(), 0.0);
    assert!(graph.topo().solids().nb() > 0);
    let mut solid = graph.shapes().reconstruct(SolidId::start().into()).unwrap();
    assert!(!solid.is_null());
    solid.set_location(location, true).unwrap();
    let original = transform_surface_props(shape).centre_of_mass().x();
    let moved = transform_surface_props(&solid).centre_of_mass().x();
    assert!((moved - (original + 50.0)).abs() <= 1.0);
}
