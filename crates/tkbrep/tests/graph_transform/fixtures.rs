fn transform_source(shape: &Shape, clear: bool) -> BRepGraph {
    let mut graph = BRepGraph::new();
    if clear {
        graph.clear().unwrap();
    }
    graph.shapes_mut().add(shape).unwrap();
    assert!(!graph.is_empty());
    graph
}
fn transform_box(clear: bool) -> BRepGraph {
    transform_source(&support::make_box(10.0, 20.0, 30.0), clear)
}
fn graph_translation(x: f64, y: f64, z: f64) -> Transform3 {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(x, y, z));
    transform
}
fn transformed_graph(
    source: &BRepGraph,
    transform: &Transform3,
    geom: GeomPolicy,
    mesh: MeshPolicy,
) -> BRepGraph {
    let mut result = BRepGraph::new();
    assert!(transform::perform(source, &mut result, transform, geom, mesh).unwrap());
    assert!(!result.is_empty());
    result
}
fn transformed_node(
    source: &BRepGraph,
    node: NodeId,
    transform: &Transform3,
    mesh: MeshPolicy,
) -> BRepGraph {
    let mut result = BRepGraph::new();
    assert!(
        transform::transform_node(source, &mut result, node, transform, GeomPolicy::Copy, mesh)
            .unwrap()
            .is_valid()
    );
    assert!(!result.is_empty());
    result
}
fn copy_transform_subgraph(source: &BRepGraph, node: NodeId) -> BRepGraph {
    let mut result = BRepGraph::new();
    assert!(
        copy::copy_node(
            source,
            &mut result,
            node,
            GeomPolicy::Copy,
            MeshPolicy::Drop,
            CachePolicy::Drop
        )
        .unwrap()
        .is_valid()
    );
    assert!(!result.is_empty());
    result
}
fn transform_near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= precision::confusion(),
        "actual {actual}, expected {expected} within Confusion"
    );
}
fn transform_point_near(actual: Point3, expected: Point3) {
    transform_near(actual.x(), expected.x());
    transform_near(actual.y(), expected.y());
    transform_near(actual.z(), expected.z());
}
fn assert_vertices_transformed(source: &BRepGraph, result: &BRepGraph, transform: &Transform3) {
    let count = source.topo().vertices().nb();
    assert_eq!(result.topo().vertices().nb(), count);
    for i in 0..count {
        let id = VertexId::new(i);
        let expected = vertex::pnt(source, id).transformed(*transform);
        transform_point_near(vertex::pnt(result, id), expected);
    }
}
fn transform_surface_props(shape: &Shape) -> GProps {
    let mut props = GProps::new();
    BRepGProp::surface_properties(shape, &mut props, false, false).unwrap();
    props
}
fn transform_face_area_sum(graph: &BRepGraph) -> f64 {
    (0..graph.topo().faces().nb())
        .map(|i| {
            let face = graph.shapes().reconstruct(FaceId::new(i).into()).unwrap();
            transform_surface_props(&face).mass().abs()
        })
        .sum()
}
fn first_transform_occurrence(graph: &BRepGraph) -> OccurrenceRefId {
    let refs = &graph
        .topo()
        .products()
        .relations(ProductId::start())
        .occurrence_ref_ids;
    assert!(!refs.is_empty());
    refs[0]
}
